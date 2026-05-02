use pgrx::pg_sys;

use crate::am::common::cost;

pub(super) unsafe extern "C-unwind" fn ec_spire_amcostestimate(
    _root: *mut pg_sys::PlannerInfo,
    path: *mut pg_sys::IndexPath,
    _loop_count: f64,
    index_startup_cost: *mut pg_sys::Cost,
    index_total_cost: *mut pg_sys::Cost,
    index_selectivity: *mut pg_sys::Selectivity,
    index_correlation: *mut f64,
    index_pages: *mut f64,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let index_info = (*path).indexinfo;
            let index_oid = (*index_info).indexoid;
            let index_relation = pg_sys::index_open(index_oid, pg_sys::NoLock as pg_sys::LOCKMODE);
            let block_count = pg_sys::RelationGetNumberOfBlocksInFork(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
            );
            pg_sys::index_close(index_relation, pg_sys::NoLock as pg_sys::LOCKMODE);

            let estimate = cost::gated_planner_cost_estimate(f64::from(block_count));
            *index_startup_cost = estimate.startup_cost;
            *index_total_cost = estimate.total_cost;
            *index_selectivity = estimate.selectivity;
            *index_correlation = estimate.correlation;
            *index_pages = estimate.index_pages;
        })
    }
}

#[cfg(feature = "pg18")]
pub(super) unsafe extern "C-unwind" fn ec_spire_amgettreeheight(_rel: pg_sys::Relation) -> i32 {
    unsafe { pgrx::pgrx_extern_c_guard(|| 0) }
}

#[cfg(feature = "pg18")]
pub(super) unsafe extern "C-unwind" fn ec_spire_amtranslatestrategy(
    strategy: pg_sys::StrategyNumber,
    _opfamily: pg_sys::Oid,
) -> pg_sys::CompareType::Type {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            match cost::amtranslatestrategy_callback(i32::from(strategy)) {
                cost::PlannerCompareType::Invalid => pg_sys::CompareType::COMPARE_INVALID,
                cost::PlannerCompareType::Lt => pg_sys::CompareType::COMPARE_LT,
                cost::PlannerCompareType::Le => pg_sys::CompareType::COMPARE_LE,
                cost::PlannerCompareType::Eq => pg_sys::CompareType::COMPARE_EQ,
                cost::PlannerCompareType::Ge => pg_sys::CompareType::COMPARE_GE,
                cost::PlannerCompareType::Gt => pg_sys::CompareType::COMPARE_GT,
                cost::PlannerCompareType::Ne => pg_sys::CompareType::COMPARE_NE,
                cost::PlannerCompareType::Overlap => pg_sys::CompareType::COMPARE_OVERLAP,
                cost::PlannerCompareType::ContainedBy => pg_sys::CompareType::COMPARE_CONTAINED_BY,
            }
        })
    }
}

#[cfg(feature = "pg18")]
pub(super) unsafe extern "C-unwind" fn ec_spire_amtranslatecmptype(
    compare_type: pg_sys::CompareType::Type,
    _opfamily: pg_sys::Oid,
) -> pg_sys::StrategyNumber {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            cost::amtranslatecmptype_callback(match compare_type {
                pg_sys::CompareType::COMPARE_LT => cost::PlannerCompareType::Lt,
                pg_sys::CompareType::COMPARE_LE => cost::PlannerCompareType::Le,
                pg_sys::CompareType::COMPARE_EQ => cost::PlannerCompareType::Eq,
                pg_sys::CompareType::COMPARE_GE => cost::PlannerCompareType::Ge,
                pg_sys::CompareType::COMPARE_GT => cost::PlannerCompareType::Gt,
                pg_sys::CompareType::COMPARE_NE => cost::PlannerCompareType::Ne,
                pg_sys::CompareType::COMPARE_OVERLAP => cost::PlannerCompareType::Overlap,
                pg_sys::CompareType::COMPARE_CONTAINED_BY => cost::PlannerCompareType::ContainedBy,
                _ => cost::PlannerCompareType::Invalid,
            }) as pg_sys::StrategyNumber
        })
    }
}
