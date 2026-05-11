//! ADR-069 DML front-door shape classification.
//!
//! The planner hook maps PostgreSQL query trees into this small input model.
//! Keeping the v1 safety rules here makes unsupported distributed DML shapes
//! fail closed before any hook can fall through to the coordinator heap path.
#![allow(dead_code)]

use pgrx::{pg_guard, pg_sys, PgList};

use std::ffi::CStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpireDmlFrontdoorOperation {
    Update,
    Delete,
    PkSelect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpireDmlFrontdoorValueKind {
    ConstBigint,
    ParamBigint,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SpireDmlFrontdoorShapeInput<'a> {
    pub(crate) operation: SpireDmlFrontdoorOperation,
    pub(crate) ec_spire_distributed_table: bool,
    pub(crate) single_table: bool,
    pub(crate) has_join: bool,
    pub(crate) has_subquery: bool,
    pub(crate) has_returning: bool,
    pub(crate) pk_column: &'a str,
    pub(crate) predicate_column: Option<&'a str>,
    pub(crate) predicate_operator: Option<&'a str>,
    pub(crate) predicate_value_kind: SpireDmlFrontdoorValueKind,
    pub(crate) updated_columns: &'a [&'a str],
    pub(crate) projected_columns: &'a [&'a str],
    pub(crate) embedding_columns: &'a [&'a str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorShapeRow {
    pub(crate) supported: bool,
    pub(crate) operation: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) status: &'static str,
    pub(crate) error: Option<&'static str>,
    pub(crate) hint: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorHookStatusRow {
    pub(crate) hook_name: &'static str,
    pub(crate) planner_hook_installed: bool,
    pub(crate) query_shape_classifier_enabled: bool,
    pub(crate) plan_rewrite_enabled: bool,
    pub(crate) status: &'static str,
    pub(crate) next_step: &'static str,
}

pub(crate) struct SpireDmlFrontdoorQueryContext<'a> {
    pub(crate) ec_spire_distributed_table: bool,
    pub(crate) pk_column: &'a str,
    pub(crate) column_names: &'a [(pg_sys::AttrNumber, &'a str)],
    pub(crate) embedding_columns: &'a [&'a str],
}

static mut PREVIOUS_PLANNER_HOOK: pg_sys::planner_hook_type = None;
static mut PLANNER_HOOK_INSTALLED: bool = false;

const ADR_069_HINT: &str = "See ADR-069 for the v1 SPIRE distributed DML shape.";

pub(crate) unsafe fn register_dml_frontdoor_planner_hook() {
    unsafe {
        if !PLANNER_HOOK_INSTALLED {
            PREVIOUS_PLANNER_HOOK = pg_sys::planner_hook;
            pg_sys::planner_hook = Some(ec_spire_dml_frontdoor_planner_hook);
            PLANNER_HOOK_INSTALLED = true;
        }
    }
}

pub(crate) fn dml_frontdoor_hook_status_row() -> SpireDmlFrontdoorHookStatusRow {
    let installed = unsafe { PLANNER_HOOK_INSTALLED };
    SpireDmlFrontdoorHookStatusRow {
        hook_name: "ec_spire_dml_frontdoor_planner_hook",
        planner_hook_installed: installed,
        query_shape_classifier_enabled: true,
        plan_rewrite_enabled: false,
        status: if installed {
            "pass_through_query_classifier_ready"
        } else {
            "not_installed"
        },
        next_step: "wire relation metadata and CustomScan executor replacement",
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_dml_frontdoor_planner_hook(
    parse: *mut pg_sys::Query,
    query_string: *const core::ffi::c_char,
    cursor_options: core::ffi::c_int,
    bound_params: pg_sys::ParamListInfo,
) -> *mut pg_sys::PlannedStmt {
    if let Some(previous_hook) = unsafe { PREVIOUS_PLANNER_HOOK } {
        unsafe { previous_hook(parse, query_string, cursor_options, bound_params) }
    } else {
        unsafe { pg_sys::standard_planner(parse, query_string, cursor_options, bound_params) }
    }
}

pub(crate) fn classify_dml_frontdoor_shape(
    input: SpireDmlFrontdoorShapeInput<'_>,
) -> SpireDmlFrontdoorShapeRow {
    let operation = operation_name(input.operation);
    if !input.ec_spire_distributed_table {
        return unsupported(
            operation,
            "not_distributed_table",
            "not an ec_spire distributed coordinator table",
            None,
        );
    }
    if !input.single_table || input.has_join {
        return unsupported_v1(
            operation,
            "unsupported_join_shape",
            "ec_spire_distributed: joined DML is not yet supported in v1",
        );
    }
    if input.has_subquery {
        return unsupported_v1(
            operation,
            "unsupported_subquery_shape",
            "ec_spire_distributed: subquery DML is not yet supported in v1",
        );
    }
    if input.has_returning {
        return unsupported_v1(
            operation,
            "unsupported_returning_shape",
            "ec_spire_distributed: RETURNING is not yet supported in v1",
        );
    }
    if input.pk_column.is_empty()
        || input.predicate_column != Some(input.pk_column)
        || input.predicate_operator != Some("=")
        || !matches!(
            input.predicate_value_kind,
            SpireDmlFrontdoorValueKind::ConstBigint | SpireDmlFrontdoorValueKind::ParamBigint
        )
    {
        return unsupported_v1(
            operation,
            "unsupported_pk_predicate",
            "ec_spire_distributed: DML requires a bigint primary-key equality predicate in v1",
        );
    }

    match input.operation {
        SpireDmlFrontdoorOperation::Update => classify_update(input),
        SpireDmlFrontdoorOperation::Delete => supported(operation, "delete_by_pk"),
        SpireDmlFrontdoorOperation::PkSelect => classify_pk_select(input),
    }
}

fn classify_update(input: SpireDmlFrontdoorShapeInput<'_>) -> SpireDmlFrontdoorShapeRow {
    let operation = operation_name(input.operation);
    if input.updated_columns.is_empty() {
        return unsupported_v1(
            operation,
            "unsupported_empty_update",
            "ec_spire_distributed: UPDATE requires at least one target column in v1",
        );
    }
    if input
        .updated_columns
        .iter()
        .any(|column| *column == input.pk_column)
    {
        return unsupported_v1(
            operation,
            "unsupported_pk_update",
            "ec_spire_distributed: UPDATE of the primary-key column is not supported in v1",
        );
    }
    if input.updated_columns.iter().any(|column| {
        input
            .embedding_columns
            .iter()
            .any(|embedding_column| embedding_column == column)
    }) {
        return unsupported(
            operation,
            "embedding_update_rejected",
            "ec_spire_distributed: UPDATE of indexed embedding column is not supported on a distributed ec_spire table. Use DELETE + INSERT.",
            Some("Cross-shard atomic moves will be available in a future release."),
        );
    }
    supported(operation, "update_non_embedding_by_pk")
}

fn classify_pk_select(input: SpireDmlFrontdoorShapeInput<'_>) -> SpireDmlFrontdoorShapeRow {
    let operation = operation_name(input.operation);
    if input.projected_columns.is_empty() {
        return unsupported_v1(
            operation,
            "unsupported_empty_projection",
            "ec_spire_distributed: PK SELECT requires at least one projected column in v1",
        );
    }
    supported(operation, "pk_select_by_pk")
}

pub(crate) unsafe fn classify_dml_frontdoor_query(
    query: *mut pg_sys::Query,
    context: SpireDmlFrontdoorQueryContext<'_>,
) -> Option<SpireDmlFrontdoorShapeRow> {
    if query.is_null() {
        return None;
    }
    let query_ref = unsafe { query.as_ref()? };
    let operation = dml_frontdoor_operation_for_query(query_ref)?;
    let range_table_ref = unsafe { single_range_table_ref(query_ref) };
    let target_rtindex = match operation {
        SpireDmlFrontdoorOperation::Update | SpireDmlFrontdoorOperation::Delete => {
            query_ref.resultRelation
        }
        SpireDmlFrontdoorOperation::PkSelect => range_table_ref.unwrap_or_default(),
    };
    let (predicate_column, predicate_operator, predicate_value_kind) =
        unsafe { dml_frontdoor_pk_predicate(query_ref, target_rtindex, &context) };
    let updated_columns = if operation == SpireDmlFrontdoorOperation::Update {
        unsafe { dml_frontdoor_target_columns(query_ref.targetList, &context) }
    } else {
        Vec::new()
    };
    let projected_columns = if operation == SpireDmlFrontdoorOperation::PkSelect {
        unsafe { dml_frontdoor_target_columns(query_ref.targetList, &context) }
    } else {
        Vec::new()
    };
    let updated_column_refs = updated_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let projected_column_refs = projected_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    Some(classify_dml_frontdoor_shape(SpireDmlFrontdoorShapeInput {
        operation,
        ec_spire_distributed_table: context.ec_spire_distributed_table,
        single_table: range_table_ref.is_some(),
        has_join: range_table_ref.is_none(),
        has_subquery: dml_frontdoor_query_has_subquery_shape(query_ref),
        has_returning: !query_ref.returningList.is_null(),
        pk_column: context.pk_column,
        predicate_column: predicate_column.as_deref(),
        predicate_operator,
        predicate_value_kind,
        updated_columns: &updated_column_refs,
        projected_columns: &projected_column_refs,
        embedding_columns: context.embedding_columns,
    }))
}

fn dml_frontdoor_operation_for_query(query: &pg_sys::Query) -> Option<SpireDmlFrontdoorOperation> {
    match query.commandType {
        pg_sys::CmdType::CMD_UPDATE => Some(SpireDmlFrontdoorOperation::Update),
        pg_sys::CmdType::CMD_DELETE => Some(SpireDmlFrontdoorOperation::Delete),
        pg_sys::CmdType::CMD_SELECT => Some(SpireDmlFrontdoorOperation::PkSelect),
        _ => None,
    }
}

fn dml_frontdoor_query_has_subquery_shape(query: &pg_sys::Query) -> bool {
    query.hasSubLinks
        || query.hasModifyingCTE
        || query.hasRecursive
        || !query.cteList.is_null()
        || !query.setOperations.is_null()
}

unsafe fn single_range_table_ref(query: &pg_sys::Query) -> Option<i32> {
    let jointree = unsafe { query.jointree.as_ref()? };
    if jointree.fromlist.is_null() {
        return None;
    }
    let fromlist = unsafe { PgList::<pg_sys::Node>::from_pg(jointree.fromlist) };
    if fromlist.len() != 1 {
        return None;
    }
    let from_node = fromlist.get_ptr(0)?;
    if from_node.is_null() || unsafe { (*from_node).type_ } != pg_sys::NodeTag::T_RangeTblRef {
        return None;
    }
    let range_table_ref = from_node.cast::<pg_sys::RangeTblRef>();
    Some(unsafe { (*range_table_ref).rtindex })
}

unsafe fn dml_frontdoor_pk_predicate(
    query: &pg_sys::Query,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> (
    Option<String>,
    Option<&'static str>,
    SpireDmlFrontdoorValueKind,
) {
    let Some(jointree) = (unsafe { query.jointree.as_ref() }) else {
        return (None, None, SpireDmlFrontdoorValueKind::Other);
    };
    let qual = jointree.quals;
    if qual.is_null() || unsafe { (*qual).type_ } != pg_sys::NodeTag::T_OpExpr {
        return (None, None, SpireDmlFrontdoorValueKind::Other);
    }
    let op_expr = qual.cast::<pg_sys::OpExpr>();
    let operator =
        if unsafe { pg_sys::get_opcode((*op_expr).opno) } == pg_sys::Oid::from(pg_sys::F_INT8EQ) {
            Some("=")
        } else {
            Some("other")
        };
    let args = unsafe { PgList::<pg_sys::Expr>::from_pg((*op_expr).args) };
    if args.len() != 2 {
        return (None, operator, SpireDmlFrontdoorValueKind::Other);
    }
    let left = args.get_ptr(0);
    let right = args.get_ptr(1);
    match (left, right) {
        (Some(left), Some(right)) => {
            if let Some(column) = unsafe { dml_frontdoor_var_column(left, target_rtindex, context) }
            {
                return (Some(column), operator, unsafe {
                    dml_frontdoor_value_kind(right)
                });
            }
            if let Some(column) =
                unsafe { dml_frontdoor_var_column(right, target_rtindex, context) }
            {
                return (Some(column), operator, unsafe {
                    dml_frontdoor_value_kind(left)
                });
            }
            (None, operator, SpireDmlFrontdoorValueKind::Other)
        }
        _ => (None, operator, SpireDmlFrontdoorValueKind::Other),
    }
}

unsafe fn dml_frontdoor_var_column(
    expr: *mut pg_sys::Expr,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> Option<String> {
    if expr.is_null() || unsafe { (*expr.cast::<pg_sys::Node>()).type_ } != pg_sys::NodeTag::T_Var {
        return None;
    }
    let var = unsafe { &*expr.cast::<pg_sys::Var>() };
    if var.varno != target_rtindex || var.varlevelsup != 0 || var.varattno <= 0 {
        return None;
    }
    context
        .column_names
        .iter()
        .find_map(|(attno, name)| (*attno == var.varattno).then(|| (*name).to_owned()))
}

unsafe fn dml_frontdoor_value_kind(expr: *mut pg_sys::Expr) -> SpireDmlFrontdoorValueKind {
    if expr.is_null() {
        return SpireDmlFrontdoorValueKind::Other;
    }
    match unsafe { (*expr.cast::<pg_sys::Node>()).type_ } {
        pg_sys::NodeTag::T_Const => {
            let const_expr = unsafe { &*expr.cast::<pg_sys::Const>() };
            if !const_expr.constisnull && const_expr.consttype == pg_sys::INT8OID {
                SpireDmlFrontdoorValueKind::ConstBigint
            } else {
                SpireDmlFrontdoorValueKind::Other
            }
        }
        pg_sys::NodeTag::T_Param => {
            let param = unsafe { &*expr.cast::<pg_sys::Param>() };
            if param.paramtype == pg_sys::INT8OID {
                SpireDmlFrontdoorValueKind::ParamBigint
            } else {
                SpireDmlFrontdoorValueKind::Other
            }
        }
        _ => SpireDmlFrontdoorValueKind::Other,
    }
}

unsafe fn dml_frontdoor_target_columns(
    target_list: *mut pg_sys::List,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> Vec<String> {
    if target_list.is_null() {
        return Vec::new();
    }
    let targets = unsafe { PgList::<pg_sys::TargetEntry>::from_pg(target_list) };
    let mut columns = Vec::new();
    for target_entry in targets.iter_ptr() {
        let Some(target_entry) = (unsafe { target_entry.as_ref() }) else {
            continue;
        };
        if target_entry.resjunk {
            continue;
        }
        if let Some(column) = context
            .column_names
            .iter()
            .find_map(|(attno, name)| (*attno == target_entry.resno).then(|| (*name).to_owned()))
        {
            columns.push(column);
            continue;
        }
        if !target_entry.resname.is_null() {
            if let Ok(column) = unsafe { CStr::from_ptr(target_entry.resname) }.to_str() {
                columns.push(column.to_owned());
            }
        }
    }
    columns
}

fn operation_name(operation: SpireDmlFrontdoorOperation) -> &'static str {
    match operation {
        SpireDmlFrontdoorOperation::Update => "update_non_embedding",
        SpireDmlFrontdoorOperation::Delete => "delete",
        SpireDmlFrontdoorOperation::PkSelect => "pk_select",
    }
}

fn supported(operation: &'static str, kind: &'static str) -> SpireDmlFrontdoorShapeRow {
    SpireDmlFrontdoorShapeRow {
        supported: true,
        operation,
        kind,
        status: "supported_v1_shape",
        error: None,
        hint: None,
    }
}

fn unsupported_v1(
    operation: &'static str,
    kind: &'static str,
    error: &'static str,
) -> SpireDmlFrontdoorShapeRow {
    unsupported(operation, kind, error, Some(ADR_069_HINT))
}

fn unsupported(
    operation: &'static str,
    kind: &'static str,
    error: &'static str,
    hint: Option<&'static str>,
) -> SpireDmlFrontdoorShapeRow {
    SpireDmlFrontdoorShapeRow {
        supported: false,
        operation,
        kind,
        status: "unsupported_shape",
        error: Some(error),
        hint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_accepts_update_delete_and_pk_select_v1_shapes() {
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&["title"], &["embedding"])).kind,
            "update_non_embedding_by_pk"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(delete_input()).kind,
            "delete_by_pk"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(select_input(&["id", "title"])).kind,
            "pk_select_by_pk"
        );
    }

    #[test]
    fn classifier_rejects_joins_subqueries_and_returning() {
        let mut joined = update_input(&["title"], &["embedding"]);
        joined.has_join = true;
        assert_eq!(
            classify_dml_frontdoor_shape(joined).kind,
            "unsupported_join_shape"
        );

        let mut subquery = delete_input();
        subquery.has_subquery = true;
        assert_eq!(
            classify_dml_frontdoor_shape(subquery).kind,
            "unsupported_subquery_shape"
        );

        let mut returning = delete_input();
        returning.has_returning = true;
        assert_eq!(
            classify_dml_frontdoor_shape(returning).kind,
            "unsupported_returning_shape"
        );
    }

    #[test]
    fn classifier_requires_bigint_pk_equality_predicate() {
        let mut wrong_column = select_input(&["id"]);
        wrong_column.predicate_column = Some("title");
        assert_eq!(
            classify_dml_frontdoor_shape(wrong_column).kind,
            "unsupported_pk_predicate"
        );

        let mut wrong_operator = select_input(&["id"]);
        wrong_operator.predicate_operator = Some(">");
        assert_eq!(
            classify_dml_frontdoor_shape(wrong_operator).kind,
            "unsupported_pk_predicate"
        );

        let mut wrong_value = select_input(&["id"]);
        wrong_value.predicate_value_kind = SpireDmlFrontdoorValueKind::Other;
        assert_eq!(
            classify_dml_frontdoor_shape(wrong_value).kind,
            "unsupported_pk_predicate"
        );
    }

    #[test]
    fn classifier_rejects_embedding_and_pk_updates() {
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&["embedding"], &["embedding"])).kind,
            "embedding_update_rejected"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&["id"], &["embedding"])).kind,
            "unsupported_pk_update"
        );
    }

    #[test]
    fn classifier_rejects_empty_update_or_projection() {
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&[], &["embedding"])).kind,
            "unsupported_empty_update"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(select_input(&[])).kind,
            "unsupported_empty_projection"
        );
    }

    #[test]
    fn query_layer_maps_command_and_subquery_flags() {
        let mut update_query = pg_sys::Query::default();
        update_query.commandType = pg_sys::CmdType::CMD_UPDATE;
        assert_eq!(
            dml_frontdoor_operation_for_query(&update_query),
            Some(SpireDmlFrontdoorOperation::Update)
        );

        let mut delete_query = pg_sys::Query::default();
        delete_query.commandType = pg_sys::CmdType::CMD_DELETE;
        assert_eq!(
            dml_frontdoor_operation_for_query(&delete_query),
            Some(SpireDmlFrontdoorOperation::Delete)
        );

        let mut select_query = pg_sys::Query::default();
        select_query.commandType = pg_sys::CmdType::CMD_SELECT;
        assert_eq!(
            dml_frontdoor_operation_for_query(&select_query),
            Some(SpireDmlFrontdoorOperation::PkSelect)
        );
        assert!(!dml_frontdoor_query_has_subquery_shape(&select_query));

        select_query.hasSubLinks = true;
        assert!(dml_frontdoor_query_has_subquery_shape(&select_query));
    }

    #[test]
    fn query_layer_recognizes_bigint_const_and_param_values() {
        let mut bigint_const = pg_sys::Const::default();
        bigint_const.xpr.type_ = pg_sys::NodeTag::T_Const;
        bigint_const.consttype = pg_sys::INT8OID;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut bigint_const as *mut pg_sys::Const).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::ConstBigint
        );

        bigint_const.constisnull = true;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut bigint_const as *mut pg_sys::Const).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::Other
        );

        let mut bigint_param = pg_sys::Param::default();
        bigint_param.xpr.type_ = pg_sys::NodeTag::T_Param;
        bigint_param.paramtype = pg_sys::INT8OID;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut bigint_param as *mut pg_sys::Param).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::ParamBigint
        );
    }

    #[test]
    fn query_layer_binds_target_relation_var_to_column_name() {
        let context = SpireDmlFrontdoorQueryContext {
            ec_spire_distributed_table: true,
            pk_column: "id",
            column_names: &[(1, "id"), (2, "title"), (3, "embedding")],
            embedding_columns: &["embedding"],
        };
        let mut var = pg_sys::Var::default();
        var.xpr.type_ = pg_sys::NodeTag::T_Var;
        var.varno = 1;
        var.varattno = 1;

        assert_eq!(
            unsafe {
                dml_frontdoor_var_column(
                    (&mut var as *mut pg_sys::Var).cast::<pg_sys::Expr>(),
                    1,
                    &context,
                )
            },
            Some("id".to_owned())
        );

        var.varno = 2;
        assert_eq!(
            unsafe {
                dml_frontdoor_var_column(
                    (&mut var as *mut pg_sys::Var).cast::<pg_sys::Expr>(),
                    1,
                    &context,
                )
            },
            None
        );
    }

    fn update_input<'a>(
        updated_columns: &'a [&'a str],
        embedding_columns: &'a [&'a str],
    ) -> SpireDmlFrontdoorShapeInput<'a> {
        base_input(
            SpireDmlFrontdoorOperation::Update,
            updated_columns,
            &[],
            embedding_columns,
        )
    }

    fn delete_input<'a>() -> SpireDmlFrontdoorShapeInput<'a> {
        base_input(SpireDmlFrontdoorOperation::Delete, &[], &[], &[])
    }

    fn select_input<'a>(projected_columns: &'a [&'a str]) -> SpireDmlFrontdoorShapeInput<'a> {
        base_input(
            SpireDmlFrontdoorOperation::PkSelect,
            &[],
            projected_columns,
            &[],
        )
    }

    fn base_input<'a>(
        operation: SpireDmlFrontdoorOperation,
        updated_columns: &'a [&'a str],
        projected_columns: &'a [&'a str],
        embedding_columns: &'a [&'a str],
    ) -> SpireDmlFrontdoorShapeInput<'a> {
        SpireDmlFrontdoorShapeInput {
            operation,
            ec_spire_distributed_table: true,
            single_table: true,
            has_join: false,
            has_subquery: false,
            has_returning: false,
            pk_column: "id",
            predicate_column: Some("id"),
            predicate_operator: Some("="),
            predicate_value_kind: SpireDmlFrontdoorValueKind::ConstBigint,
            updated_columns,
            projected_columns,
            embedding_columns,
        }
    }
}
