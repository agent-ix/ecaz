use pgrx::pg_sys;
use std::collections::BTreeSet;
use std::ffi::c_void;

use crate::am::common::pg_ptr::{pg_list as cs_pg_list, pg_ref as cs_pg_ref};

const PLAN_ORDERING_PROOF_NONE: u32 = 0;
const PLAN_ORDERING_PROOF_EXACT: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PayloadFallbackReason {
    WholeRow,
    UnprovedExpression,
    QueryValueRelationVar,
    VisibilityCheckRequiresPayload,
}

impl PayloadFallbackReason {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::WholeRow => "whole_row",
            Self::UnprovedExpression => "unproved_expression",
            Self::QueryValueRelationVar => "query_value_relation_var",
            Self::VisibilityCheckRequiresPayload => "visibility_check_requires_payload",
        }
    }
}

/// A proved payload contract, kept distinct from a conservative all-column
/// fallback even when both resolve to the same physical attnum set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PayloadAttributeMask {
    Exact(Vec<pg_sys::AttrNumber>),
    AllColumns(PayloadFallbackReason),
}

impl PayloadAttributeMask {
    pub(crate) fn variant_label(&self) -> &'static str {
        match self {
            Self::Exact(_) => "exact",
            Self::AllColumns(_) => "all_columns",
        }
    }

    pub(crate) fn exact_attnums(&self) -> Option<&[pg_sys::AttrNumber]> {
        match self {
            Self::Exact(attnums) => Some(attnums),
            Self::AllColumns(_) => None,
        }
    }
}

/// The sole ORDER BY target entry is safe to omit from the payload walk only
/// after the caller has proved that pathkeys, query shape, and upper consumers
/// do not require its resjunk value.
#[derive(Clone, Copy)]
pub(crate) struct OrderingOnlyProof {
    pub(crate) scan_relid: pg_sys::Index,
    pub(crate) relation_attnum: pg_sys::AttrNumber,
    pub(crate) operator_oid: pg_sys::Oid,
}

/// Resolve a Var to this base relation's heap attnum before or after setrefs.
/// PlanCustomPath may receive INDEX_VAR Vars for a projection-capable custom
/// path; their stable syntactic identity retains the base rel/attribute pair.
pub(crate) unsafe fn relation_var_attno(
    expression: *mut pg_sys::Expr,
    scan_relid: pg_sys::Index,
) -> Option<pg_sys::AttrNumber> {
    let node = cs_pg_ref(expression.cast::<pg_sys::Node>())?;
    if node.type_ != pg_sys::NodeTag::T_Var {
        return None;
    }
    let var = cs_pg_ref(expression.cast::<pg_sys::Var>())?;
    if var.varlevelsup != 0 {
        return None;
    }
    if u32::try_from(var.varno).ok() == Some(scan_relid) {
        return Some(var.varattno);
    }
    if var.varno == pg_sys::INDEX_VAR && var.varnosyn == scan_relid {
        return Some(var.varattnosyn);
    }
    None
}

struct AttributeWalkerContext {
    scan_relid: pg_sys::Index,
    attnums: BTreeSet<pg_sys::AttrNumber>,
    fallback: Option<PayloadFallbackReason>,
}

unsafe extern "C-unwind" fn collect_relation_attributes(
    node: *mut pg_sys::Node,
    context: *mut c_void,
) -> bool {
    if node.is_null() || context.is_null() {
        return false;
    }
    let context = &mut *context.cast::<AttributeWalkerContext>();
    if (*node).type_ == pg_sys::NodeTag::T_Var {
        if let Some(attnum) = relation_var_attno(node.cast::<pg_sys::Expr>(), context.scan_relid) {
            if attnum < 0 {
                pgrx::error!(
                    "EC_UNSUPPORTED_PROJECTION: multi-node ec_distann scan references system attribute {}",
                    attnum
                );
            }
            if attnum == 0 {
                context.fallback = Some(PayloadFallbackReason::WholeRow);
                return true;
            }
            context.attnums.insert(attnum);
        }
        return false;
    }
    pg_sys::expression_tree_walker(
        node,
        Some(collect_relation_attributes),
        context as *mut _ as *mut c_void,
    )
}

unsafe fn walk_expression(
    expression: *mut pg_sys::Expr,
    context: &mut AttributeWalkerContext,
) -> Result<(), PayloadFallbackReason> {
    if expression.is_null() {
        return Err(PayloadFallbackReason::UnprovedExpression);
    }
    let stopped = collect_relation_attributes(
        expression.cast::<pg_sys::Node>(),
        context as *mut _ as *mut c_void,
    );
    if let Some(reason) = context.fallback {
        return Err(reason);
    }
    if stopped {
        return Err(PayloadFallbackReason::UnprovedExpression);
    }
    Ok(())
}

unsafe fn is_ordering_only_target(
    entry: &pg_sys::TargetEntry,
    proof: OrderingOnlyProof,
    query_value: *mut pg_sys::Expr,
) -> bool {
    if entry.expr.is_null() || query_value.is_null() {
        return false;
    }
    let Some(planned_node) = cs_pg_ref(entry.expr.cast::<pg_sys::Node>()) else {
        return false;
    };
    if planned_node.type_ != pg_sys::NodeTag::T_OpExpr {
        return false;
    }
    let Some(planned_op) = cs_pg_ref(entry.expr.cast::<pg_sys::OpExpr>()) else {
        return false;
    };
    if planned_op.opno != proof.operator_oid {
        return false;
    }
    let planned_args = cs_pg_list::<pg_sys::Expr>(planned_op.args);
    if planned_args.len() != 2 {
        return false;
    }
    for relation_index in 0..2 {
        let value_index = 1 - relation_index;
        let Some(planned_relation) = planned_args.get_ptr(relation_index) else {
            continue;
        };
        let planned_attnum = relation_var_attno(planned_relation, proof.scan_relid);
        if planned_attnum != Some(proof.relation_attnum) {
            continue;
        }
        let Some(planned_value) = planned_args.get_ptr(value_index) else {
            continue;
        };
        if pg_sys::equal(planned_value.cast(), query_value.cast()) {
            return true;
        }
    }
    false
}

/// Replace the one proven-unused ordering projection with a typed NULL. The
/// upper Limit shares this target list but does not project or consume the junk
/// value; avoiding evaluation is required when its vector is not transported.
pub(crate) unsafe fn elide_ordering_only_target(
    target_list: *mut pg_sys::List,
    query_value: *mut pg_sys::Expr,
    proof: OrderingOnlyProof,
) -> bool {
    let mut matching = cs_pg_list::<pg_sys::TargetEntry>(target_list)
        .iter_ptr()
        .filter_map(|entry| cs_pg_ref(entry).map(|entry_ref| (entry, entry_ref)))
        .filter(|(_, entry)| is_ordering_only_target(entry, proof, query_value))
        .map(|(entry, _)| entry)
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return false;
    }
    let entry = &mut *matching[0];
    let expression = entry.expr.cast::<pg_sys::Node>();
    let null_expression = pg_sys::makeNullConst(
        pg_sys::exprType(expression),
        pg_sys::exprTypmod(expression),
        pg_sys::exprCollation(expression),
    );
    if null_expression.is_null() {
        return false;
    }
    entry.expr = null_expression.cast::<pg_sys::Expr>();
    true
}

/// Derive the complete set of base-relation attributes the CustomScan executor
/// may read. The three input trees are the plan target list, plan quals, and the
/// Const/Param ORDER BY query value stored in custom_exprs[0].
pub(crate) unsafe fn derive_payload_attribute_mask(
    scan_relid: pg_sys::Index,
    target_list: *mut pg_sys::List,
    quals: *mut pg_sys::List,
    query_value: *mut pg_sys::Expr,
    ordering_only: Option<OrderingOnlyProof>,
) -> PayloadAttributeMask {
    let target_entries = cs_pg_list::<pg_sys::TargetEntry>(target_list);
    let skipped_ordering_entry = ordering_only.and_then(|proof| {
        let matches = target_entries
            .iter_ptr()
            .filter_map(|entry| cs_pg_ref(entry))
            .filter(|entry| is_ordering_only_target(entry, proof, query_value))
            .count();
        (matches == 1).then_some(proof)
    });

    let mut context = AttributeWalkerContext {
        scan_relid,
        attnums: BTreeSet::new(),
        fallback: None,
    };
    for entry in target_entries.iter_ptr() {
        let Some(entry) = cs_pg_ref(entry) else {
            return PayloadAttributeMask::AllColumns(PayloadFallbackReason::UnprovedExpression);
        };
        if skipped_ordering_entry
            .is_some_and(|proof| is_ordering_only_target(entry, proof, query_value))
        {
            continue;
        }
        if let Err(reason) = walk_expression(entry.expr, &mut context) {
            return PayloadAttributeMask::AllColumns(reason);
        }
    }

    for qual in cs_pg_list::<pg_sys::Expr>(quals).iter_ptr() {
        if let Err(reason) = walk_expression(qual, &mut context) {
            return PayloadAttributeMask::AllColumns(reason);
        }
    }

    let supported_query_shape = cs_pg_ref(query_value.cast::<pg_sys::Node>()).is_some_and(|node| {
        matches!(
            node.type_,
            pg_sys::NodeTag::T_Const | pg_sys::NodeTag::T_Param
        )
    });
    if !supported_query_shape {
        return PayloadAttributeMask::AllColumns(PayloadFallbackReason::UnprovedExpression);
    }

    // Walk the query value independently. Comparing the main set's size would
    // miss a forbidden relation Var when that same attnum was already required
    // by the target list or quals.
    let mut query_context = AttributeWalkerContext {
        scan_relid,
        attnums: BTreeSet::new(),
        fallback: None,
    };
    if walk_expression(query_value, &mut query_context).is_err()
        || !query_context.attnums.is_empty()
    {
        return PayloadAttributeMask::AllColumns(PayloadFallbackReason::QueryValueRelationVar);
    }

    let attnums = context.attnums.into_iter().collect::<Vec<_>>();
    if attnums.is_empty() {
        // A remote row-tier fetch currently supplies the tombstone/visibility
        // decision as well as values. Task 222 classifies but does not implement
        // a zero-payload/index-only path.
        PayloadAttributeMask::AllColumns(PayloadFallbackReason::VisibilityCheckRequiresPayload)
    } else {
        PayloadAttributeMask::Exact(attnums)
    }
}

/// Append the plan-time ordering-only exemption proof to CustomScan private.
pub(crate) unsafe fn append_ordering_proof_plan_private(
    mut list: *mut pg_sys::List,
    proof: Option<OrderingOnlyProof>,
) -> *mut pg_sys::List {
    if let Some(proof) = proof {
        list = pg_sys::lappend_oid(list, pg_sys::Oid::from(PLAN_ORDERING_PROOF_EXACT));
        let attnum = u32::try_from(i32::from(proof.relation_attnum)).unwrap_or_else(|_| {
            pgrx::error!(
                "EcDistannDistributedScan ordering proof contains invalid attnum {}",
                proof.relation_attnum
            )
        });
        list = pg_sys::lappend_oid(list, pg_sys::Oid::from(attnum));
        list = pg_sys::lappend_oid(list, proof.operator_oid);
    } else {
        list = pg_sys::lappend_oid(list, pg_sys::Oid::from(PLAN_ORDERING_PROOF_NONE));
        list = pg_sys::lappend_oid(list, pg_sys::InvalidOid);
        list = pg_sys::lappend_oid(list, pg_sys::InvalidOid);
    }
    list
}

/// Decode the ordering proof stored after mode/index/top-k in custom_private.
pub(crate) unsafe fn ordering_proof_from_plan_private(
    list: *mut pg_sys::List,
    scan_relid: pg_sys::Index,
) -> Result<Option<OrderingOnlyProof>, String> {
    let len = pg_sys::list_length(list);
    if len != 6 {
        return Err(format!("ordering proof private length {len} is not 6"));
    }
    let tag = u32::from(pg_sys::list_nth_oid(list, 3));
    match tag {
        PLAN_ORDERING_PROOF_NONE => Ok(None),
        PLAN_ORDERING_PROOF_EXACT => {
            let raw_attnum = u32::from(pg_sys::list_nth_oid(list, 4));
            let relation_attnum = pg_sys::AttrNumber::try_from(raw_attnum)
                .ok()
                .filter(|attnum| *attnum > 0)
                .ok_or_else(|| format!("ordering proof attnum {raw_attnum} is invalid"))?;
            let operator_oid = pg_sys::list_nth_oid(list, 5);
            if operator_oid == pg_sys::InvalidOid {
                return Err("ordering proof operator OID is invalid".to_owned());
            }
            Ok(Some(OrderingOnlyProof {
                scan_relid,
                relation_attnum,
                operator_oid,
            }))
        }
        _ => Err(format!("ordering proof tag {tag} is invalid")),
    }
}
