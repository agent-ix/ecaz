#[derive(Debug, Clone, Copy, PartialEq)]
struct SpireCustomScanCostEstimate {
    startup_cost: f64,
    total_cost: f64,
}

#[derive(Clone, Copy)]
struct CustomScanPlannerRel<'a> {
    root_ref: &'a pg_sys::PlannerInfo,
    rel_ref: &'a pg_sys::RelOptInfo,
}

impl<'a> CustomScanPlannerRel<'a> {
    unsafe fn new(root: *mut pg_sys::PlannerInfo, rel: *mut pg_sys::RelOptInfo) -> Option<Self> {
        // SAFETY: caller guarantees root/rel are the live planner callback
        // pointers for immediate inspection during this callback.
        let (root_ref, rel_ref) = unsafe { (custom_scan_pg_ref(root)?, custom_scan_pg_ref(rel)?) };
        Some(Self { root_ref, rel_ref })
    }

    unsafe fn new_base_rel(
        root: *mut pg_sys::PlannerInfo,
        rel: *mut pg_sys::RelOptInfo,
        rte: *mut pg_sys::RangeTblEntry,
    ) -> Option<Self> {
        // SAFETY: caller guarantees root/rel/rte are live planner hook
        // pointers; retaining root/rel and validating rte here ties later safe
        // accessors to this callback-owned view instead of arbitrary pointers.
        let planner_rel = unsafe { Self::new(root, rel)? };
        let _rte = unsafe { custom_scan_pg_ref(rte)? };
        Some(planner_rel)
    }

    fn is_plain_base_relation(self) -> bool {
        self.rel_ref.reloptkind == pg_sys::RelOptKind::RELOPT_BASEREL
            && self.rel_ref.rtekind == pg_sys::RTEKind::RTE_RELATION
    }

    fn has_vector_order_limit_shape(self) -> bool {
        !self.root_ref.sort_pathkeys.is_null() && self.root_ref.limit_tuples >= 0.0
    }

    fn top_k(self) -> Option<usize> {
        if self.root_ref.limit_tuples < 0.0 || !self.root_ref.limit_tuples.is_finite() {
            return None;
        }
        Some(self.root_ref.limit_tuples.max(0.0).ceil() as usize)
    }

    fn orderby_query_expr(self) -> Option<*mut pg_sys::Expr> {
        // SAFETY: this view was built from live planner callback pointers and
        // the referenced Query/List nodes are inspected immediately.
        unsafe {
            let query = custom_scan_pg_ref(self.root_ref.parse)?;
            if query.sortClause.is_null() || query.targetList.is_null() {
                return None;
            }
            let sort_clauses = custom_scan_pg_list::<pg_sys::SortGroupClause>(query.sortClause);
            if sort_clauses.len() != 1 {
                return None;
            }
            let sort_clause = custom_scan_pg_ref(sort_clauses.get_ptr(0)?)?;
            let target_list = custom_scan_pg_list::<pg_sys::TargetEntry>(query.targetList);
            for target_entry in target_list.iter_ptr() {
                let Some(target_entry) = custom_scan_pg_ref(target_entry) else {
                    continue;
                };
                if target_entry.ressortgroupref != sort_clause.tleSortGroupRef {
                    continue;
                }
                return self.query_expr_from_sort_expr(target_entry.expr);
            }
            None
        }
    }

    fn query_expr_from_sort_expr(self, expr: *mut pg_sys::Expr) -> Option<*mut pg_sys::Expr> {
        // SAFETY: expr comes from the planner-owned target list reached through
        // this live callback view and is only inspected for node/list shape.
        unsafe {
            let op_expr = CustomScanExpr::new(expr)?.op_expr()?;
            let args = custom_scan_pg_list::<pg_sys::Expr>(op_expr.args);
            if args.len() != 2 {
                return None;
            }
            let left_ptr = args.get_ptr(0)?;
            let right_ptr = args.get_ptr(1)?;
            let left = CustomScanExpr::new(left_ptr)?;
            let right = CustomScanExpr::new(right_ptr)?;
            if left.is_relation_var(self.rel_ref.relid) && right.is_query_value() {
                return Some(right_ptr);
            }
            if right.is_relation_var(self.rel_ref.relid) && left.is_query_value() {
                return Some(left_ptr);
            }
            None
        }
    }

    fn for_each_index_info(self, mut visit: impl FnMut(&pg_sys::IndexOptInfo) -> bool) {
        // SAFETY: indexlist is owned by the live RelOptInfo represented by this
        // view and each index node is inspected synchronously.
        unsafe {
            let index_list = custom_scan_pg_list::<pg_sys::IndexOptInfo>(self.rel_ref.indexlist);
            for index_info in index_list.iter_ptr() {
                let Some(index_info) = custom_scan_pg_ref(index_info) else {
                    continue;
                };
                if !visit(index_info) {
                    break;
                }
            }
        }
    }

    fn sort_pathkeys(self) -> *mut pg_sys::List {
        self.root_ref.sort_pathkeys
    }

    fn reltarget(self) -> *mut pg_sys::PathTarget {
        self.rel_ref.reltarget
    }

    fn baserestrictinfo(self) -> *mut pg_sys::List {
        self.rel_ref.baserestrictinfo
    }

    fn output_rows_for_custom_scan(self) -> f64 {
        if self.root_ref.limit_tuples >= 0.0 {
            self.root_ref.limit_tuples.max(1.0)
        } else {
            self.rel_ref.rows.max(1.0)
        }
    }

    fn target_width(self) -> f64 {
        // SAFETY: reltarget, when non-null, is owned by the live RelOptInfo
        // represented by this planner callback view.
        unsafe { custom_scan_pg_ref(self.rel_ref.reltarget) }
            .map(|target| f64::from(target.width.max(0)))
            .unwrap_or(0.0)
    }

    fn estimate_custom_scan_cost(
        self,
        output_rows: f64,
        eligibility: &SpireCustomScanIndexEligibilityRow,
    ) -> SpireCustomScanCostEstimate {
        // SAFETY: this view exists only inside the PostgreSQL planner callback
        // where backend-local planner cost globals are valid to read.
        unsafe {
            estimate_custom_scan_cost(
                output_rows,
                self.rel_ref.rows.max(1.0),
                self.target_width(),
                eligibility,
            )
        }
    }
}

#[derive(Clone, Copy)]
struct CustomScanExpr<'a> {
    expr: *mut pg_sys::Expr,
    node_ref: &'a pg_sys::Node,
}

impl<'a> CustomScanExpr<'a> {
    unsafe fn new(expr: *mut pg_sys::Expr) -> Option<Self> {
        // SAFETY: caller guarantees expr, when non-null, is a live PostgreSQL
        // expression node for immediate tag-checked inspection.
        let node_ref = unsafe { custom_scan_pg_ref(expr.cast::<pg_sys::Node>())? };
        Some(Self { expr, node_ref })
    }

    fn tag(self) -> pg_sys::NodeTag {
        self.node_ref.type_
    }

    fn op_expr(self) -> Option<&'a pg_sys::OpExpr> {
        if self.tag() != pg_sys::NodeTag::T_OpExpr {
            return None;
        }
        // SAFETY: the node tag was checked before casting to OpExpr.
        unsafe { custom_scan_pg_ref(self.expr.cast::<pg_sys::OpExpr>()) }
    }

    fn var(self) -> Option<&'a pg_sys::Var> {
        if self.tag() != pg_sys::NodeTag::T_Var {
            return None;
        }
        // SAFETY: the node tag was checked before casting to Var.
        unsafe { custom_scan_pg_ref(self.expr.cast::<pg_sys::Var>()) }
    }

    fn const_expr(self) -> Option<&'a pg_sys::Const> {
        if self.tag() != pg_sys::NodeTag::T_Const {
            return None;
        }
        // SAFETY: the node tag was checked before casting to Const.
        unsafe { custom_scan_pg_ref(self.expr.cast::<pg_sys::Const>()) }
    }

    fn param(self) -> Option<&'a pg_sys::Param> {
        if self.tag() != pg_sys::NodeTag::T_Param {
            return None;
        }
        // SAFETY: the node tag was checked before casting to Param.
        unsafe { custom_scan_pg_ref(self.expr.cast::<pg_sys::Param>()) }
    }

    fn is_relation_var(self, relid: pg_sys::Index) -> bool {
        self.var()
            .map(|var| u32::try_from(var.varno).ok() == Some(relid) && var.varlevelsup == 0)
            .unwrap_or(false)
    }

    fn query_values_from_const(self) -> Option<Vec<f32>> {
        let const_ref = self.const_expr()?;
        // SAFETY: the node tag was checked above. Float4 array
        // type/nullability are checked before decoding the datum without
        // taking ownership from PostgreSQL.
        let values = unsafe {
            if const_ref.constisnull || const_ref.consttype != pg_sys::FLOAT4ARRAYOID {
                return None;
            }
            Vec::<f32>::from_polymorphic_datum(const_ref.constvalue, false, pg_sys::FLOAT4ARRAYOID)?
        };
        if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
            return None;
        }
        Some(values)
    }

    fn is_query_value(self) -> bool {
        match self.tag() {
            pg_sys::NodeTag::T_Const => self.query_values_from_const().is_some(),
            pg_sys::NodeTag::T_Param => self
                .param()
                .map(|param| param.paramtype == pg_sys::FLOAT4ARRAYOID)
                .unwrap_or(false),
            _ => false,
        }
    }
}

unsafe fn estimate_custom_scan_cost(
    output_rows: f64,
    rel_rows: f64,
    target_width: f64,
    eligibility: &SpireCustomScanIndexEligibilityRow,
) -> SpireCustomScanCostEstimate {
    let constants = current_planner_cost_constants();
    let cpu_tuple_cost = current_cpu_tuple_cost();
    estimate_custom_scan_cost_with_constants(
        output_rows,
        rel_rows,
        target_width,
        eligibility,
        constants,
        cpu_tuple_cost,
    )
}

fn estimate_custom_scan_cost_with_constants(
    output_rows: f64,
    rel_rows: f64,
    target_width: f64,
    eligibility: &SpireCustomScanIndexEligibilityRow,
    constants: PlannerCostConstants,
    cpu_tuple_cost: f64,
) -> SpireCustomScanCostEstimate {
    let output_rows = output_rows.max(1.0);
    let rel_rows = rel_rows.max(output_rows);
    let fanout = eligibility.remote_available_node_count.max(1) as f64;
    let remote_placements = eligibility.remote_available_placement_count.max(1) as f64;
    let routing_scores = remote_placements.min(CUSTOM_SCAN_ROUTING_SCORE_BOUND);
    let routing_traversal_cost = routing_scores * constants.cpu_operator_cost;
    let remote_dispatch_cost =
        fanout * CUSTOM_SCAN_REMOTE_DISPATCH_CPU_UNITS * constants.cpu_operator_cost;
    let bounded_heap_rows = (output_rows * fanout).min(rel_rows);
    let heap_rerank_cost = bounded_heap_rows * (cpu_tuple_cost + constants.cpu_operator_cost);
    let merge_cost = output_rows
        * fanout.log2().max(1.0)
        * CUSTOM_SCAN_MERGE_CPU_UNITS
        * constants.cpu_operator_cost;
    let tuple_delivery_cost = output_rows * cpu_tuple_cost;
    let tuple_width_cost = output_rows
        * target_width.max(0.0)
        * CUSTOM_SCAN_TUPLE_BYTE_CPU_UNITS
        * constants.cpu_operator_cost;
    let startup_cost = routing_traversal_cost + remote_dispatch_cost;
    SpireCustomScanCostEstimate {
        startup_cost,
        total_cost: startup_cost
            + heap_rerank_cost
            + merge_cost
            + tuple_delivery_cost
            + tuple_width_cost,
    }
}

unsafe fn custom_scan_pg_list<T>(list: *mut pg_sys::List) -> PgList<T> {
    // SAFETY: callers pass PostgreSQL-owned planner lists and consume the view
    // immediately during the current planner callback.
    unsafe { PgList::<T>::from_pg(list) }
}

unsafe fn custom_scan_pg_ref<'a, T>(ptr: *mut T) -> Option<&'a T> {
    // SAFETY: callers pass PostgreSQL planner pointers that are live for the
    // current callback and copy or inspect the referenced fields immediately.
    unsafe { ptr.as_ref() }
}

unsafe fn custom_scan_current_relation(
    node: *mut pg_sys::CustomScanState,
    label: &str,
) -> pg_sys::Relation {
    // SAFETY: caller guarantees node is the live CustomScanState for the
    // current executor callback.
    let Some(node) = (unsafe { custom_scan_pg_ref(node) }) else {
        pgrx::error!("EcSpireDistributedScan {label} missing custom scan state");
    };
    let relation = node.ss.ss_currentRelation;
    if relation.is_null() {
        pgrx::error!("EcSpireDistributedScan {label} missing scan relation");
    }
    relation
}
