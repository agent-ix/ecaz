#[derive(Debug, Clone, Copy, PartialEq)]
struct SpireCustomScanCostEstimate {
    startup_cost: f64,
    total_cost: f64,
}

unsafe fn estimate_custom_scan_cost(
    output_rows: f64,
    rel_rows: f64,
    target_width: f64,
    eligibility: &SpireCustomScanIndexEligibilityRow,
) -> SpireCustomScanCostEstimate {
    // SAFETY: planner cost constants and cpu_tuple_cost are backend-local GUC
    // reads during planner path construction.
    let (constants, cpu_tuple_cost) =
        unsafe { (current_planner_cost_constants(), pg_sys::cpu_tuple_cost) };
    estimate_custom_scan_cost_with_constants(
        output_rows,
        rel_rows,
        target_width,
        eligibility,
        constants,
        cpu_tuple_cost,
    )
}

fn custom_scan_target_width(target: *mut pg_sys::PathTarget) -> f64 {
    custom_scan_pg_ref(target)
        .map(|target| f64::from(target.width.max(0)))
        .unwrap_or(0.0)
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

unsafe fn custom_scan_top_k(root: *mut pg_sys::PlannerInfo) -> Option<usize> {
    let root_ref = custom_scan_pg_ref(root)?;
    if root_ref.limit_tuples < 0.0 || !root_ref.limit_tuples.is_finite() {
        return None;
    }
    Some(root_ref.limit_tuples.max(0.0).ceil() as usize)
}

unsafe fn custom_scan_orderby_query_expr(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
) -> Option<*mut pg_sys::Expr> {
    let root_ref = custom_scan_pg_ref(root)?;
    let rel_ref = custom_scan_pg_ref(rel)?;
    let query = custom_scan_pg_ref(root_ref.parse)?;
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
        return custom_scan_query_expr_from_sort_expr(target_entry.expr, rel_ref.relid);
    }
    None
}

fn custom_scan_query_expr_from_sort_expr(
    expr: *mut pg_sys::Expr,
    relid: pg_sys::Index,
) -> Option<*mut pg_sys::Expr> {
    let op_expr = custom_scan_op_expr(expr)?;
    let args = custom_scan_pg_list::<pg_sys::Expr>(op_expr.args);
    if args.len() != 2 {
        return None;
    }
    let left = args.get_ptr(0)?;
    let right = args.get_ptr(1)?;
    if custom_scan_expr_is_relation_var(left, relid) && custom_scan_expr_is_query_value(right) {
        return Some(right);
    }
    if custom_scan_expr_is_relation_var(right, relid) && custom_scan_expr_is_query_value(left) {
        return Some(left);
    }
    None
}

fn custom_scan_expr_is_relation_var(expr: *mut pg_sys::Expr, relid: pg_sys::Index) -> bool {
    custom_scan_var_expr(expr)
        .map(|var| u32::try_from(var.varno).ok() == Some(relid) && var.varlevelsup == 0)
        .unwrap_or(false)
}

fn custom_scan_pg_list<T>(list: *mut pg_sys::List) -> PgList<T> {
    // SAFETY: callers pass PostgreSQL-owned planner lists and consume the view
    // immediately during the current planner callback.
    unsafe { PgList::<T>::from_pg(list) }
}

fn custom_scan_pg_ref<'a, T>(ptr: *mut T) -> Option<&'a T> {
    // SAFETY: callers pass PostgreSQL planner pointers that are live for the
    // current callback and copy or inspect the referenced fields immediately.
    unsafe { ptr.as_ref() }
}

fn custom_scan_op_expr<'a>(expr: *mut pg_sys::Expr) -> Option<&'a pg_sys::OpExpr> {
    if custom_scan_expr_node_tag(expr)? != pg_sys::NodeTag::T_OpExpr {
        return None;
    }
    custom_scan_pg_ref(expr.cast::<pg_sys::OpExpr>())
}

fn custom_scan_var_expr<'a>(expr: *mut pg_sys::Expr) -> Option<&'a pg_sys::Var> {
    if custom_scan_expr_node_tag(expr)? != pg_sys::NodeTag::T_Var {
        return None;
    }
    custom_scan_pg_ref(expr.cast::<pg_sys::Var>())
}

fn custom_scan_expr_node_tag(expr: *mut pg_sys::Expr) -> Option<pg_sys::NodeTag> {
    let node = custom_scan_pg_ref(expr.cast::<pg_sys::Node>())?;
    Some(node.type_)
}
