pub(crate) unsafe fn custom_scan_index_eligibility_row(
    index_relation: pg_sys::Relation,
) -> SpireCustomScanIndexEligibilityRow {
    unsafe {
        custom_scan_index_eligibility_result(index_relation).unwrap_or_else(|e| pgrx::error!("{e}"))
    }
}

unsafe fn custom_scan_index_eligibility_result(
    index_relation: pg_sys::Relation,
) -> Result<SpireCustomScanIndexEligibilityRow, String> {
    let root_control = unsafe { super::page::read_root_control_page(index_relation) };
    if root_control.active_epoch == 0 {
        return Ok(SpireCustomScanIndexEligibilityRow {
            active_epoch: 0,
            local_placement_count: 0,
            remote_node_count: 0,
            remote_available_node_count: 0,
            remote_placement_count: 0,
            remote_available_placement_count: 0,
            remote_unavailable_placement_count: 0,
            all_remote_placements_available: false,
            eligible_for_custom_scan: false,
            status: "no_active_epoch",
            next_step: "keep local-only ec_spire index AM path",
        });
    }

    let placement_directory =
        unsafe { load_custom_scan_placement_directory(index_relation, root_control)? };
    let active_epoch = root_control.active_epoch;
    let mut local_placement_count = 0_u64;
    let mut remote_placement_count = 0_u64;
    let mut remote_available_placement_count = 0_u64;
    let mut remote_unavailable_placement_count = 0_u64;
    let mut remote_node_ids = std::collections::BTreeSet::new();
    let mut remote_available_node_ids = std::collections::BTreeSet::new();

    for placement in placement_directory.entries {
        if placement.node_id == meta::SPIRE_LOCAL_NODE_ID {
            local_placement_count = local_placement_count.saturating_add(1);
        } else {
            remote_node_ids.insert(placement.node_id);
            remote_placement_count = remote_placement_count.saturating_add(1);
            if placement.state == meta::SpirePlacementState::Available {
                remote_available_placement_count =
                    remote_available_placement_count.saturating_add(1);
                remote_available_node_ids.insert(placement.node_id);
            } else {
                remote_unavailable_placement_count =
                    remote_unavailable_placement_count.saturating_add(1);
            }
        }
    }

    let eligible = active_epoch != 0 && remote_available_placement_count > 0;
    let all_remote_placements_available =
        remote_placement_count > 0 && remote_unavailable_placement_count == 0;
    Ok(SpireCustomScanIndexEligibilityRow {
        active_epoch,
        local_placement_count,
        remote_node_count: remote_node_ids.len() as u64,
        remote_available_node_count: remote_available_node_ids.len() as u64,
        remote_placement_count,
        remote_available_placement_count,
        remote_unavailable_placement_count,
        all_remote_placements_available,
        eligible_for_custom_scan: eligible,
        status: if eligible {
            "customscan_candidate"
        } else if active_epoch == 0 {
            "no_active_epoch"
        } else if remote_placement_count == 0 {
            "local_only"
        } else {
            "no_available_remote_placements"
        },
        next_step: if eligible {
            "planner path generation must also verify ORDER BY vector distance LIMIT query shape"
        } else {
            "keep local-only ec_spire index AM path"
        },
    })
}

unsafe fn load_custom_scan_placement_directory(
    index_relation: pg_sys::Relation,
    root_control: meta::SpireRootControlState,
) -> Result<meta::SpirePlacementDirectory, String> {
    // The SQL eligibility wrapper normally returns `no_active_epoch` before
    // this helper is called. Keep the helper fail-closed so future callers
    // cannot accidentally dereference an empty placement-directory TID.
    if root_control.active_epoch == 0 {
        return Err("ec_spire cannot load placement directory for empty active epoch".to_owned());
    }

    // ADR-067 planner eligibility needs only placement availability. Avoid the
    // heavier fanout loader used by executor paths, which also decodes epoch
    // and object manifests; executor paths remain responsible for full
    // identity and manifest validation before result-stream merge.
    let placement_bytes = unsafe {
        super::page::read_object_tuple(index_relation, root_control.placement_directory_tid)?
    };
    let placement_directory = meta::SpirePlacementDirectory::decode(&placement_bytes)?;
    if placement_directory.epoch != root_control.active_epoch {
        return Err(format!(
            "ec_spire root/control active epoch {} does not match placement directory {}",
            root_control.active_epoch, placement_directory.epoch
        ));
    }
    Ok(placement_directory)
}

struct IndexScanGuard {
    scan: pg_sys::IndexScanDesc,
}

impl IndexScanGuard {
    fn begin(
        relation: pg_sys::Relation,
        index: pg_sys::Relation,
        snapshot: pg_sys::Snapshot,
    ) -> Option<Self> {
        #[cfg(feature = "pg18")]
        // SAFETY: `relation`, `index`, and `snapshot` are owned by live guards
        // in the caller; this guard owns the matching `index_endscan`.
        let scan =
            unsafe { pg_sys::index_beginscan(relation, index, snapshot, ptr::null_mut(), 1, 0) };
        #[cfg(not(feature = "pg18"))]
        // SAFETY: `relation`, `index`, and `snapshot` are owned by live guards
        // in the caller; this guard owns the matching `index_endscan`.
        let scan = unsafe { pg_sys::index_beginscan(relation, index, snapshot, 1, 0) };
        if scan.is_null() {
            return None;
        }
        Some(Self { scan })
    }

    fn as_ptr(&self) -> pg_sys::IndexScanDesc {
        self.scan
    }
}

impl Drop for IndexScanGuard {
    fn drop(&mut self) {
        // SAFETY: `scan` was returned by `index_beginscan` in
        // `IndexScanGuard::begin`; this guard owns the matching end call.
        unsafe { pg_sys::index_endscan(self.scan) };
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_set_rel_pathlist_hook(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rti: pg_sys::Index,
    rte: *mut pg_sys::RangeTblEntry,
) {
    unsafe {
        if let Some(previous_hook) = PREVIOUS_SET_REL_PATHLIST_HOOK {
            previous_hook(root, rel, rti, rte);
        }
    }
    if let Some((index_oid, eligibility)) =
        unsafe { custom_scan_candidate_index_oid(root, rel, rte) }
    {
        unsafe { add_custom_scan_path(root, rel, index_oid, eligibility) };
    }
    if let Some(index_oid) = unsafe { dml_pk_select_candidate_index_oid(root, rel, rte) } {
        unsafe { add_dml_pk_select_custom_scan_path(root, rel, index_oid) };
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_plan_custom_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    best_path: *mut pg_sys::CustomPath,
    tlist: *mut pg_sys::List,
    clauses: *mut pg_sys::List,
    custom_plans: *mut pg_sys::List,
) -> *mut pg_sys::Plan {
    unsafe {
        let mode = custom_scan_mode_from_path(best_path).unwrap_or_else(|| {
            pgrx::error!("EcSpireDistributedScan CustomPath is missing plan mode")
        });
        if mode.is_dml() {
            return plan_dml_custom_path(root, rel, best_path, tlist, clauses, custom_plans, mode);
        }

        let top_k = custom_scan_top_k(root).unwrap_or(1);
        let query_expr = custom_scan_orderby_query_expr(root, rel).unwrap_or_else(|| {
            pgrx::error!(
                "EcSpireDistributedScan could not extract ORDER BY vector query expression"
            )
        });
        let custom_exprs = pg_sys::lappend(
            std::ptr::null_mut(),
            pg_sys::copyObjectImpl(query_expr.cast()).cast(),
        );

        let mut custom_scan =
            PgBox::<pg_sys::CustomScan>::alloc_node(pg_sys::NodeTag::T_CustomScan);
        custom_scan.scan.plan.type_ = pg_sys::NodeTag::T_CustomScan;
        custom_scan.scan.plan.disabled_nodes = (*best_path).path.disabled_nodes;
        custom_scan.scan.plan.startup_cost = (*best_path).path.startup_cost;
        custom_scan.scan.plan.total_cost = (*best_path).path.total_cost;
        custom_scan.scan.plan.plan_rows = (*best_path).path.rows;
        custom_scan.scan.plan.plan_width = if !(*best_path).path.pathtarget.is_null() {
            (*(*best_path).path.pathtarget).width
        } else {
            0
        };
        custom_scan.scan.plan.parallel_aware = false;
        custom_scan.scan.plan.parallel_safe = false;
        custom_scan.scan.plan.async_capable = false;
        custom_scan.scan.plan.targetlist = tlist;
        custom_scan.scan.plan.qual = pg_sys::extract_actual_clauses(clauses, false);
        custom_scan.scan.scanrelid = (*(*best_path).path.parent).relid;
        custom_scan.flags = (*best_path).flags;
        custom_scan.custom_plans = custom_plans;
        custom_scan.custom_exprs = custom_exprs;
        custom_scan.custom_private = pg_sys::lappend_oid(
            pg_sys::lappend_oid(
                pg_sys::lappend_oid(
                    std::ptr::null_mut(),
                    pg_sys::Oid::from(CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT),
                ),
                custom_scan_index_oid_from_path(best_path),
            ),
            pg_sys::Oid::from(u32::try_from(top_k).unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan LIMIT exceeds CustomScan plan-private range")
            })),
        );
        custom_scan.custom_scan_tlist = std::ptr::null_mut();
        custom_scan.custom_relids = std::ptr::null_mut();
        custom_scan.methods = &raw const CUSTOM_SCAN_METHODS;
        custom_scan.into_pg() as *mut pg_sys::Plan
    }
}

unsafe fn plan_dml_custom_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    best_path: *mut pg_sys::CustomPath,
    tlist: *mut pg_sys::List,
    clauses: *mut pg_sys::List,
    custom_plans: *mut pg_sys::List,
    mode: SpireCustomScanPlanMode,
) -> *mut pg_sys::Plan {
    unsafe {
        let plan_expr = match super::dml_frontdoor_primitive_plan_expr_from_baserel(root, rel)
            .unwrap_or_else(|| {
                pgrx::error!("EcSpireDistributedScan could not build DML expression handoff")
            }) {
            Ok(plan_expr) => plan_expr,
            Err(err) => pgrx::error!("{err}"),
        };
        let plan_mode = custom_scan_plan_mode_for_dml_mode(plan_expr.primitive_plan.mode);
        if plan_mode != mode {
            pgrx::error!(
                "EcSpireDistributedScan DML plan mode {:?} does not match primitive mode {:?}",
                mode,
                plan_expr.primitive_plan.mode
            )
        }
        let custom_exprs = custom_scan_dml_custom_exprs_from_plan_expr(&plan_expr);
        let mut custom_scan =
            PgBox::<pg_sys::CustomScan>::alloc_node(pg_sys::NodeTag::T_CustomScan);
        custom_scan.scan.plan.type_ = pg_sys::NodeTag::T_CustomScan;
        custom_scan.scan.plan.disabled_nodes = (*best_path).path.disabled_nodes;
        custom_scan.scan.plan.startup_cost = (*best_path).path.startup_cost;
        custom_scan.scan.plan.total_cost = (*best_path).path.total_cost;
        custom_scan.scan.plan.plan_rows = (*best_path).path.rows;
        custom_scan.scan.plan.plan_width = if !(*best_path).path.pathtarget.is_null() {
            (*(*best_path).path.pathtarget).width
        } else {
            0
        };
        custom_scan.scan.plan.parallel_aware = false;
        custom_scan.scan.plan.parallel_safe = false;
        custom_scan.scan.plan.async_capable = false;
        custom_scan.scan.plan.targetlist = tlist;
        custom_scan.scan.plan.qual = pg_sys::extract_actual_clauses(clauses, false);
        custom_scan.scan.scanrelid = (*(*best_path).path.parent).relid;
        custom_scan.flags = (*best_path).flags;
        custom_scan.custom_plans = custom_plans;
        custom_scan.custom_exprs = custom_exprs;
        custom_scan.custom_private = custom_scan_dml_plan_private(
            mode,
            custom_scan_index_oid_from_path(best_path),
            &plan_expr.primitive_plan.pk_argument.pk_column,
            &plan_expr.primitive_plan.updated_columns,
            &plan_expr.primitive_plan.projected_columns,
        );
        custom_scan.custom_scan_tlist = std::ptr::null_mut();
        custom_scan.custom_relids = std::ptr::null_mut();
        custom_scan.methods = &raw const CUSTOM_SCAN_METHODS;
        custom_scan.into_pg() as *mut pg_sys::Plan
    }
}

unsafe fn custom_scan_dml_custom_exprs_from_plan_expr(
    plan_expr: &super::dml_frontdoor::SpireDmlFrontdoorPrimitivePlanExpr,
) -> *mut pg_sys::List {
    unsafe {
        let mut custom_exprs = pg_sys::lappend(
            std::ptr::null_mut(),
            pg_sys::copyObjectImpl(plan_expr.pk_value_expr.cast()).cast(),
        );
        for expr in &plan_expr.updated_value_exprs {
            custom_exprs =
                pg_sys::lappend(custom_exprs, pg_sys::copyObjectImpl((*expr).cast()).cast());
        }
        custom_exprs
    }
}

pub(crate) unsafe fn custom_scan_dml_replacement_plan(
    plan_expr: super::dml_frontdoor::SpireDmlFrontdoorPrimitivePlanExpr,
    fallback_plan: *mut pg_sys::Plan,
) -> *mut pg_sys::Plan {
    unsafe {
        let mode = custom_scan_plan_mode_for_dml_mode(plan_expr.primitive_plan.mode);
        let custom_exprs = custom_scan_dml_custom_exprs_from_plan_expr(&plan_expr);
        let mut custom_scan =
            PgBox::<pg_sys::CustomScan>::alloc_node(pg_sys::NodeTag::T_CustomScan);
        custom_scan.scan.plan.type_ = pg_sys::NodeTag::T_CustomScan;
        // This replacement is not competing in path selection; the planner
        // has already produced fallback_plan. Copy its cost fields so EXPLAIN
        // remains roughly comparable until DML-specific costing exists.
        custom_scan.scan.plan.disabled_nodes = if fallback_plan.is_null() {
            0
        } else {
            (*fallback_plan).disabled_nodes
        };
        custom_scan.scan.plan.startup_cost = if fallback_plan.is_null() {
            0.0
        } else {
            (*fallback_plan).startup_cost
        };
        custom_scan.scan.plan.total_cost = if fallback_plan.is_null() {
            0.0
        } else {
            (*fallback_plan).total_cost
        };
        custom_scan.scan.plan.plan_rows = 0.0;
        custom_scan.scan.plan.plan_width = 0;
        custom_scan.scan.plan.parallel_aware = false;
        custom_scan.scan.plan.parallel_safe = false;
        custom_scan.scan.plan.async_capable = false;
        custom_scan.scan.plan.targetlist = std::ptr::null_mut();
        custom_scan.scan.plan.qual = std::ptr::null_mut();
        custom_scan.scan.scanrelid = 0;
        custom_scan.flags = 0;
        custom_scan.custom_plans = std::ptr::null_mut();
        custom_scan.custom_exprs = custom_exprs;
        custom_scan.custom_private = custom_scan_dml_plan_private(
            mode,
            plan_expr.primitive_plan.index_oid,
            &plan_expr.primitive_plan.pk_argument.pk_column,
            &plan_expr.primitive_plan.updated_columns,
            &plan_expr.primitive_plan.projected_columns,
        );
        custom_scan.custom_scan_tlist = std::ptr::null_mut();
        custom_scan.custom_relids = std::ptr::null_mut();
        custom_scan.methods = &raw const CUSTOM_SCAN_METHODS;
        custom_scan.into_pg() as *mut pg_sys::Plan
    }
}

unsafe fn custom_scan_candidate_index_oid(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rte: *mut pg_sys::RangeTblEntry,
) -> Option<(pg_sys::Oid, SpireCustomScanIndexEligibilityRow)> {
    if root.is_null() || rel.is_null() || rte.is_null() {
        return None;
    }
    let rel_ref = unsafe { rel.as_ref()? };
    if rel_ref.reloptkind != pg_sys::RelOptKind::RELOPT_BASEREL {
        return None;
    }
    if rel_ref.rtekind != pg_sys::RTEKind::RTE_RELATION {
        return None;
    }
    let root_ref = unsafe { root.as_ref()? };
    if root_ref.sort_pathkeys.is_null() || root_ref.limit_tuples < 0.0 {
        return None;
    }
    let _ = unsafe { custom_scan_orderby_query_expr(root, rel)? };

    let ec_spire_am_oid = unsafe { pg_sys::get_index_am_oid(EC_SPIRE_AM_NAME.as_ptr(), true) };
    if ec_spire_am_oid == pg_sys::InvalidOid {
        return None;
    }

    let index_list = unsafe { PgList::<pg_sys::IndexOptInfo>::from_pg(rel_ref.indexlist) };
    for index_info in index_list.iter_ptr() {
        let Some(index_info) = (unsafe { index_info.as_ref() }) else {
            continue;
        };
        if index_info.relam != ec_spire_am_oid {
            continue;
        }
        let Some(index_relation) =
            crate::storage::relation_guard::IndexRelationGuard::try_access_share(
                index_info.indexoid,
            )
        else {
            continue;
        };
        let eligibility = unsafe { custom_scan_index_eligibility_result(index_relation.as_ptr()) };
        if let Ok(row) = eligibility {
            if row.eligible_for_custom_scan {
                return Some((index_info.indexoid, row));
            }
        }
    }
    None
}

unsafe fn dml_pk_select_candidate_index_oid(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rte: *mut pg_sys::RangeTblEntry,
) -> Option<pg_sys::Oid> {
    if root.is_null() || rel.is_null() || rte.is_null() {
        return None;
    }
    let rel_ref = unsafe { rel.as_ref()? };
    if rel_ref.reloptkind != pg_sys::RelOptKind::RELOPT_BASEREL {
        return None;
    }
    if rel_ref.rtekind != pg_sys::RTEKind::RTE_RELATION {
        return None;
    }
    let ec_spire_am_oid = unsafe { pg_sys::get_index_am_oid(EC_SPIRE_AM_NAME.as_ptr(), true) };
    if ec_spire_am_oid == pg_sys::InvalidOid {
        return None;
    }

    let index_list = unsafe { PgList::<pg_sys::IndexOptInfo>::from_pg(rel_ref.indexlist) };
    let mut placement_index_oid = None;
    for index_info in index_list.iter_ptr() {
        let Some(index_info) = (unsafe { index_info.as_ref() }) else {
            continue;
        };
        if index_info.relam == ec_spire_am_oid
            && unsafe { custom_scan_index_has_sql_placement(index_info.indexoid) }
        {
            placement_index_oid = Some(index_info.indexoid);
            break;
        }
    }
    let placement_index_oid = placement_index_oid?;
    let plan_expr = match unsafe {
        super::dml_frontdoor_pk_select_primitive_plan_expr_from_baserel(root, rel)?
    } {
        Ok(plan_expr) => plan_expr,
        Err(_err) => return None,
    };
    if plan_expr.primitive_plan.mode
        != super::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload
    {
        return None;
    }
    (plan_expr.primitive_plan.index_oid == placement_index_oid)
        .then_some(plan_expr.primitive_plan.index_oid)
}

unsafe fn custom_scan_index_has_sql_placement(index_oid: pg_sys::Oid) -> bool {
    unsafe {
        let placement_oid = pg_sys::RelnameGetRelid(EC_SPIRE_PLACEMENT_RELNAME.as_ptr());
        let placement_by_index_oid =
            pg_sys::RelnameGetRelid(EC_SPIRE_PLACEMENT_BY_INDEX_OID_RELNAME.as_ptr());
        if placement_oid == pg_sys::InvalidOid {
            return false;
        }
        if placement_by_index_oid == pg_sys::InvalidOid {
            return false;
        }
        let Some(placement_relation) =
            crate::storage::relation_guard::HeapRelationGuard::try_access_share(placement_oid)
        else {
            return false;
        };
        let Some(placement_index) =
            crate::storage::relation_guard::IndexRelationGuard::try_access_share(
                placement_by_index_oid,
            )
        else {
            return false;
        };

        let mut scan_key = std::mem::MaybeUninit::<pg_sys::ScanKeyData>::zeroed().assume_init();
        pg_sys::ScanKeyInit(
            &mut scan_key,
            EC_SPIRE_PLACEMENT_INDEX_OID_ATTNO,
            pg_sys::BTEqualStrategyNumber as pg_sys::StrategyNumber,
            pg_sys::F_OIDEQ.into(),
            index_oid.into(),
        );
        let Some(snapshot) = crate::storage::snapshot_guard::ActiveSnapshotGuard::latest() else {
            return false;
        };
        let Some(scan) = IndexScanGuard::begin(
            placement_relation.as_ptr(),
            placement_index.as_ptr(),
            snapshot.as_ptr(),
        ) else {
            return false;
        };
        let Some(slot) =
            crate::storage::slot_guard::TupleTableSlotGuard::create(placement_relation.as_ptr())
        else {
            return false;
        };
        pg_sys::index_rescan(scan.as_ptr(), &mut scan_key, 1, ptr::null_mut(), 0);
        pg_sys::index_getnext_slot(
            scan.as_ptr(),
            pg_sys::ScanDirection::ForwardScanDirection,
            slot.as_ptr(),
        )
    }
}

unsafe fn add_custom_scan_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    index_oid: pg_sys::Oid,
    eligibility: SpireCustomScanIndexEligibilityRow,
) {
    if root.is_null() || rel.is_null() {
        return;
    }
    let root_ref = unsafe { root.as_ref().expect("checked root pointer") };
    let rel_ref = unsafe { rel.as_ref().expect("checked rel pointer") };
    let mut custom_path =
        unsafe { PgBox::<pg_sys::CustomPath>::alloc_node(pg_sys::NodeTag::T_CustomPath) };
    let rows = if root_ref.limit_tuples >= 0.0 {
        root_ref.limit_tuples.max(1.0)
    } else {
        rel_ref.rows.max(1.0)
    };
    let target_width = custom_scan_target_width(rel_ref.reltarget);
    let cost = unsafe {
        estimate_custom_scan_cost(rows, rel_ref.rows.max(1.0), target_width, &eligibility)
    };
    custom_path.path.type_ = pg_sys::NodeTag::T_CustomPath;
    custom_path.path.pathtype = pg_sys::NodeTag::T_CustomScan;
    custom_path.path.parent = rel;
    custom_path.path.pathtarget = rel_ref.reltarget;
    custom_path.path.param_info = std::ptr::null_mut();
    custom_path.path.parallel_aware = false;
    custom_path.path.parallel_safe = false;
    custom_path.path.parallel_workers = 0;
    custom_path.path.rows = rows;
    custom_path.path.disabled_nodes = 0;
    custom_path.path.startup_cost = cost.startup_cost;
    custom_path.path.total_cost = cost.total_cost;
    custom_path.path.pathkeys = root_ref.sort_pathkeys;
    custom_path.flags = pg_sys::CUSTOMPATH_SUPPORT_PROJECTION;
    custom_path.custom_paths = std::ptr::null_mut();
    custom_path.custom_restrictinfo = rel_ref.baserestrictinfo;
    custom_path.custom_private = unsafe {
        pg_sys::lappend_oid(
            pg_sys::lappend_oid(
                std::ptr::null_mut(),
                pg_sys::Oid::from(CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT),
            ),
            index_oid,
        )
    };
    custom_path.methods = &raw const CUSTOM_PATH_METHODS;

    unsafe { pg_sys::add_path(rel, custom_path.into_pg() as *mut pg_sys::Path) };
}

unsafe fn add_dml_pk_select_custom_scan_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    index_oid: pg_sys::Oid,
) {
    if root.is_null() || rel.is_null() {
        return;
    }
    let rel_ref = unsafe { rel.as_ref().expect("checked rel pointer") };
    let mut custom_path =
        unsafe { PgBox::<pg_sys::CustomPath>::alloc_node(pg_sys::NodeTag::T_CustomPath) };
    custom_path.path.type_ = pg_sys::NodeTag::T_CustomPath;
    custom_path.path.pathtype = pg_sys::NodeTag::T_CustomScan;
    custom_path.path.parent = rel;
    custom_path.path.pathtarget = rel_ref.reltarget;
    custom_path.path.param_info = std::ptr::null_mut();
    custom_path.path.parallel_aware = false;
    custom_path.path.parallel_safe = false;
    custom_path.path.parallel_workers = 0;
    custom_path.path.rows = 1.0;
    custom_path.path.disabled_nodes = 0;
    custom_path.path.startup_cost = -1.0;
    custom_path.path.total_cost = -1.0;
    custom_path.path.pathkeys = std::ptr::null_mut();
    custom_path.flags = pg_sys::CUSTOMPATH_SUPPORT_PROJECTION;
    custom_path.custom_paths = std::ptr::null_mut();
    custom_path.custom_restrictinfo = rel_ref.baserestrictinfo;
    custom_path.custom_private = unsafe {
        pg_sys::lappend_oid(
            pg_sys::lappend_oid(
                std::ptr::null_mut(),
                pg_sys::Oid::from(CUSTOM_SCAN_PLAN_MODE_DML_PK_SELECT),
            ),
            index_oid,
        )
    };
    custom_path.methods = &raw const CUSTOM_PATH_METHODS;

    unsafe { pg_sys::add_path(rel, custom_path.into_pg() as *mut pg_sys::Path) };
}
