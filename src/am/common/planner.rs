use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, Ordering};
use std::sync::OnceLock;

use pgrx::pg_sys;

const NO_COST_BITS: u64 = 0x7ff8_0000_0000_0000;
const NONE_I32: i32 = -1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlannerPathlistSnapshot {
    pub hook_registered: bool,
    pub observed: bool,
    pub relid: u32,
    pub consider_parallel: bool,
    pub rel_parallel_workers: i32,
    pub ec_hnsw_index_count: i32,
    pub amcanparallel_seen: bool,
    pub path_count: i32,
    pub index_path_count: i32,
    pub ec_hnsw_index_path_count: i32,
    pub partial_path_count: i32,
    pub partial_index_path_count: i32,
    pub partial_ec_hnsw_index_path_count: i32,
    pub best_plain_ec_hnsw_startup_cost: Option<f64>,
    pub best_plain_ec_hnsw_total_cost: Option<f64>,
    pub best_plain_ec_hnsw_parallel_workers: Option<i32>,
    pub best_plain_ec_hnsw_pathkeys: Option<i32>,
    pub best_partial_ec_hnsw_startup_cost: Option<f64>,
    pub best_partial_ec_hnsw_total_cost: Option<f64>,
    pub best_partial_ec_hnsw_parallel_workers: Option<i32>,
    pub best_partial_ec_hnsw_parallel_aware: Option<bool>,
    pub best_partial_ec_hnsw_pathkeys: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PathSummary {
    startup_cost: f64,
    total_cost: f64,
    parallel_workers: i32,
    parallel_aware: bool,
    pathkeys: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct PathlistCounters {
    path_count: i32,
    index_path_count: i32,
    ec_hnsw_index_path_count: i32,
    partial_path_count: i32,
    partial_index_path_count: i32,
    partial_ec_hnsw_index_path_count: i32,
    best_plain_ec_hnsw: Option<PathSummary>,
    best_partial_ec_hnsw: Option<PathSummary>,
}

static PREVIOUS_SET_REL_PATHLIST_HOOK: OnceLock<pg_sys::set_rel_pathlist_hook_type> =
    OnceLock::new();
static ECAZ_PLANNER_HOOK_REGISTERED: AtomicBool = AtomicBool::new(false);
static EC_HNSW_AM_OID: AtomicU32 = AtomicU32::new(0);

static OBSERVED: AtomicBool = AtomicBool::new(false);
static RELID: AtomicU32 = AtomicU32::new(0);
static CONSIDER_PARALLEL: AtomicBool = AtomicBool::new(false);
static REL_PARALLEL_WORKERS: AtomicI32 = AtomicI32::new(0);
static EC_HNSW_INDEX_COUNT: AtomicI32 = AtomicI32::new(0);
static AMCANPARALLEL_SEEN: AtomicBool = AtomicBool::new(false);
static PATH_COUNT: AtomicI32 = AtomicI32::new(0);
static INDEX_PATH_COUNT: AtomicI32 = AtomicI32::new(0);
static EC_HNSW_INDEX_PATH_COUNT: AtomicI32 = AtomicI32::new(0);
static PARTIAL_PATH_COUNT: AtomicI32 = AtomicI32::new(0);
static PARTIAL_INDEX_PATH_COUNT: AtomicI32 = AtomicI32::new(0);
static PARTIAL_EC_HNSW_INDEX_PATH_COUNT: AtomicI32 = AtomicI32::new(0);
static BEST_PLAIN_STARTUP_COST: AtomicU64 = AtomicU64::new(NO_COST_BITS);
static BEST_PLAIN_TOTAL_COST: AtomicU64 = AtomicU64::new(NO_COST_BITS);
static BEST_PLAIN_PARALLEL_WORKERS: AtomicI32 = AtomicI32::new(NONE_I32);
static BEST_PLAIN_PATHKEYS: AtomicI32 = AtomicI32::new(NONE_I32);
static BEST_PARTIAL_STARTUP_COST: AtomicU64 = AtomicU64::new(NO_COST_BITS);
static BEST_PARTIAL_TOTAL_COST: AtomicU64 = AtomicU64::new(NO_COST_BITS);
static BEST_PARTIAL_PARALLEL_WORKERS: AtomicI32 = AtomicI32::new(NONE_I32);
static BEST_PARTIAL_PARALLEL_AWARE: AtomicI32 = AtomicI32::new(NONE_I32);
static BEST_PARTIAL_PATHKEYS: AtomicI32 = AtomicI32::new(NONE_I32);

pub(crate) fn reset_planner_pathlist_snapshot() {
    OBSERVED.store(false, Ordering::Relaxed);
    RELID.store(0, Ordering::Relaxed);
    CONSIDER_PARALLEL.store(false, Ordering::Relaxed);
    REL_PARALLEL_WORKERS.store(0, Ordering::Relaxed);
    EC_HNSW_INDEX_COUNT.store(0, Ordering::Relaxed);
    AMCANPARALLEL_SEEN.store(false, Ordering::Relaxed);
    PATH_COUNT.store(0, Ordering::Relaxed);
    INDEX_PATH_COUNT.store(0, Ordering::Relaxed);
    EC_HNSW_INDEX_PATH_COUNT.store(0, Ordering::Relaxed);
    PARTIAL_PATH_COUNT.store(0, Ordering::Relaxed);
    PARTIAL_INDEX_PATH_COUNT.store(0, Ordering::Relaxed);
    PARTIAL_EC_HNSW_INDEX_PATH_COUNT.store(0, Ordering::Relaxed);
    store_optional_cost(&BEST_PLAIN_STARTUP_COST, None);
    store_optional_cost(&BEST_PLAIN_TOTAL_COST, None);
    BEST_PLAIN_PARALLEL_WORKERS.store(NONE_I32, Ordering::Relaxed);
    BEST_PLAIN_PATHKEYS.store(NONE_I32, Ordering::Relaxed);
    store_optional_cost(&BEST_PARTIAL_STARTUP_COST, None);
    store_optional_cost(&BEST_PARTIAL_TOTAL_COST, None);
    BEST_PARTIAL_PARALLEL_WORKERS.store(NONE_I32, Ordering::Relaxed);
    BEST_PARTIAL_PARALLEL_AWARE.store(NONE_I32, Ordering::Relaxed);
    BEST_PARTIAL_PATHKEYS.store(NONE_I32, Ordering::Relaxed);
}

pub(crate) fn planner_pathlist_snapshot() -> PlannerPathlistSnapshot {
    PlannerPathlistSnapshot {
        hook_registered: ECAZ_PLANNER_HOOK_REGISTERED.load(Ordering::Acquire),
        observed: OBSERVED.load(Ordering::Relaxed),
        relid: RELID.load(Ordering::Relaxed),
        consider_parallel: CONSIDER_PARALLEL.load(Ordering::Relaxed),
        rel_parallel_workers: REL_PARALLEL_WORKERS.load(Ordering::Relaxed),
        ec_hnsw_index_count: EC_HNSW_INDEX_COUNT.load(Ordering::Relaxed),
        amcanparallel_seen: AMCANPARALLEL_SEEN.load(Ordering::Relaxed),
        path_count: PATH_COUNT.load(Ordering::Relaxed),
        index_path_count: INDEX_PATH_COUNT.load(Ordering::Relaxed),
        ec_hnsw_index_path_count: EC_HNSW_INDEX_PATH_COUNT.load(Ordering::Relaxed),
        partial_path_count: PARTIAL_PATH_COUNT.load(Ordering::Relaxed),
        partial_index_path_count: PARTIAL_INDEX_PATH_COUNT.load(Ordering::Relaxed),
        partial_ec_hnsw_index_path_count: PARTIAL_EC_HNSW_INDEX_PATH_COUNT.load(Ordering::Relaxed),
        best_plain_ec_hnsw_startup_cost: load_optional_cost(&BEST_PLAIN_STARTUP_COST),
        best_plain_ec_hnsw_total_cost: load_optional_cost(&BEST_PLAIN_TOTAL_COST),
        best_plain_ec_hnsw_parallel_workers: load_optional_i32(&BEST_PLAIN_PARALLEL_WORKERS),
        best_plain_ec_hnsw_pathkeys: load_optional_i32(&BEST_PLAIN_PATHKEYS),
        best_partial_ec_hnsw_startup_cost: load_optional_cost(&BEST_PARTIAL_STARTUP_COST),
        best_partial_ec_hnsw_total_cost: load_optional_cost(&BEST_PARTIAL_TOTAL_COST),
        best_partial_ec_hnsw_parallel_workers: load_optional_i32(&BEST_PARTIAL_PARALLEL_WORKERS),
        best_partial_ec_hnsw_parallel_aware: load_optional_bool(&BEST_PARTIAL_PARALLEL_AWARE),
        best_partial_ec_hnsw_pathkeys: load_optional_i32(&BEST_PARTIAL_PATHKEYS),
    }
}

fn store_optional_cost(counter: &AtomicU64, value: Option<f64>) {
    counter.store(
        value.map(f64::to_bits).unwrap_or(NO_COST_BITS),
        Ordering::Relaxed,
    );
}

fn load_optional_cost(counter: &AtomicU64) -> Option<f64> {
    let bits = counter.load(Ordering::Relaxed);
    if bits == NO_COST_BITS {
        None
    } else {
        Some(f64::from_bits(bits))
    }
}

fn load_optional_i32(counter: &AtomicI32) -> Option<i32> {
    let value = counter.load(Ordering::Relaxed);
    if value == NONE_I32 {
        None
    } else {
        Some(value)
    }
}

fn load_optional_bool(counter: &AtomicI32) -> Option<bool> {
    match counter.load(Ordering::Relaxed) {
        NONE_I32 => None,
        0 => Some(false),
        _ => Some(true),
    }
}

unsafe fn ec_hnsw_am_oid() -> Option<pg_sys::Oid> {
    let cached = EC_HNSW_AM_OID.load(Ordering::Acquire);
    if cached != 0 {
        return Some(pg_sys::Oid::from(cached));
    }

    let oid = unsafe { pg_sys::get_am_oid(c"ec_hnsw".as_ptr(), true) };
    if oid == pg_sys::InvalidOid {
        return None;
    }

    EC_HNSW_AM_OID.store(oid.to_u32(), Ordering::Release);
    Some(oid)
}

unsafe fn index_info_is_ec_hnsw(index_info: *mut pg_sys::IndexOptInfo) -> bool {
    if index_info.is_null() {
        return false;
    }

    let Some(ec_hnsw_am_oid) = (unsafe { ec_hnsw_am_oid() }) else {
        return false;
    };
    unsafe { (*index_info).relam == ec_hnsw_am_oid }
}

unsafe fn list_length(list: *mut pg_sys::List) -> i32 {
    if list.is_null() {
        0
    } else {
        unsafe { pg_sys::list_length(list) }
    }
}

unsafe fn summarize_path(path: *mut pg_sys::Path) -> PathSummary {
    PathSummary {
        startup_cost: unsafe { (*path).startup_cost },
        total_cost: unsafe { (*path).total_cost },
        parallel_workers: unsafe { (*path).parallel_workers },
        parallel_aware: unsafe { (*path).parallel_aware },
        pathkeys: unsafe { list_length((*path).pathkeys) },
    }
}

fn choose_best_by_total(current: &mut Option<PathSummary>, candidate: PathSummary) {
    if current
        .map(|best| candidate.total_cost < best.total_cost)
        .unwrap_or(true)
    {
        *current = Some(candidate);
    }
}

fn parallel_cost_divisor(parallel_workers: i32, leader_participation: bool) -> Option<f64> {
    if parallel_workers <= 0 {
        return None;
    }

    let mut divisor = f64::from(parallel_workers);
    if leader_participation {
        let leader_contribution = 1.0 - (0.3 * f64::from(parallel_workers));
        if leader_contribution > 0.0 {
            divisor += leader_contribution;
        }
    }
    Some(divisor)
}

fn discounted_parallel_am_cost(
    index_total_cost: f64,
    parallel_workers: i32,
    leader_participation: bool,
) -> Option<f64> {
    if !index_total_cost.is_finite() || index_total_cost <= 0.0 {
        return None;
    }

    let divisor = parallel_cost_divisor(parallel_workers, leader_participation)?;
    if divisor <= 1.0 {
        return None;
    }

    Some(index_total_cost / divisor)
}

unsafe fn discount_partial_ec_hnsw_index_path_cost(index_path: *mut pg_sys::IndexPath) {
    if index_path.is_null() {
        return;
    }

    let path = unsafe { &mut (*index_path).path };
    if !path.parallel_aware || path.parallel_workers <= 0 {
        return;
    }

    let original_index_total_cost = unsafe { (*index_path).indextotalcost };
    let Some(discounted_index_total_cost) = (unsafe {
        discounted_parallel_am_cost(
            original_index_total_cost,
            path.parallel_workers,
            pg_sys::parallel_leader_participation,
        )
    }) else {
        return;
    };
    let discount = original_index_total_cost - discounted_index_total_cost;

    // `cost_index` cannot parallel-discount AM run cost because the AM cost
    // callback runs before it assigns `parallel_workers`. The PG18 pathlist
    // hook sees the completed partial IndexPath before Gather/Gather Merge
    // paths are generated, so it can correct only the parallel ec_hnsw path.
    unsafe {
        (*index_path).indextotalcost = discounted_index_total_cost;
    }
    path.total_cost = (path.total_cost - discount).max(path.startup_cost);
}

unsafe fn count_ec_hnsw_indexes(rel: *mut pg_sys::RelOptInfo) -> (i32, bool) {
    let mut count = 0;
    let mut amcanparallel_seen = false;
    let list = unsafe { (*rel).indexlist };
    if list.is_null() {
        return (count, amcanparallel_seen);
    }
    let mut cell = unsafe { pg_sys::list_head(list) };
    while !cell.is_null() {
        let index_info = unsafe { (*cell).ptr_value.cast::<pg_sys::IndexOptInfo>() };
        if unsafe { index_info_is_ec_hnsw(index_info) } {
            count += 1;
            amcanparallel_seen |= unsafe { (*index_info).amcanparallel };
        }
        cell = unsafe { pg_sys::lnext(list, cell) };
    }
    (count, amcanparallel_seen)
}

unsafe fn count_paths(list: *mut pg_sys::List, partial: bool, counters: &mut PathlistCounters) {
    if list.is_null() {
        return;
    }

    let mut cell = unsafe { pg_sys::list_head(list) };
    while !cell.is_null() {
        let path = unsafe { (*cell).ptr_value.cast::<pg_sys::Path>() };
        if !path.is_null() {
            if partial {
                counters.partial_path_count += 1;
            } else {
                counters.path_count += 1;
            }

            if unsafe { (*path).type_ } == pg_sys::NodeTag::T_IndexPath {
                if partial {
                    counters.partial_index_path_count += 1;
                } else {
                    counters.index_path_count += 1;
                }

                let index_path = path.cast::<pg_sys::IndexPath>();
                let index_info = unsafe { (*index_path).indexinfo };
                if unsafe { index_info_is_ec_hnsw(index_info) } {
                    if partial {
                        unsafe { discount_partial_ec_hnsw_index_path_cost(index_path) };
                    }
                    let summary = unsafe { summarize_path(path) };
                    if partial {
                        counters.partial_ec_hnsw_index_path_count += 1;
                        choose_best_by_total(&mut counters.best_partial_ec_hnsw, summary);
                    } else {
                        counters.ec_hnsw_index_path_count += 1;
                        choose_best_by_total(&mut counters.best_plain_ec_hnsw, summary);
                    }
                }
            }
        }
        cell = unsafe { pg_sys::lnext(list, cell) };
    }
}

fn store_path_summary(
    summary: Option<PathSummary>,
    startup_cost: &AtomicU64,
    total_cost: &AtomicU64,
    parallel_workers: &AtomicI32,
    parallel_aware: Option<&AtomicI32>,
    pathkeys: &AtomicI32,
) {
    match summary {
        Some(summary) => {
            store_optional_cost(startup_cost, Some(summary.startup_cost));
            store_optional_cost(total_cost, Some(summary.total_cost));
            parallel_workers.store(summary.parallel_workers, Ordering::Relaxed);
            if let Some(parallel_aware) = parallel_aware {
                parallel_aware.store(i32::from(summary.parallel_aware), Ordering::Relaxed);
            }
            pathkeys.store(summary.pathkeys, Ordering::Relaxed);
        }
        None => {
            store_optional_cost(startup_cost, None);
            store_optional_cost(total_cost, None);
            parallel_workers.store(NONE_I32, Ordering::Relaxed);
            if let Some(parallel_aware) = parallel_aware {
                parallel_aware.store(NONE_I32, Ordering::Relaxed);
            }
            pathkeys.store(NONE_I32, Ordering::Relaxed);
        }
    }
}

unsafe fn record_rel_pathlist(rel: *mut pg_sys::RelOptInfo) {
    if rel.is_null() || unsafe { (*rel).reloptkind } != pg_sys::RelOptKind::RELOPT_BASEREL {
        return;
    }

    let (ec_hnsw_index_count, amcanparallel_seen) = unsafe { count_ec_hnsw_indexes(rel) };
    if ec_hnsw_index_count == 0 {
        return;
    }

    let mut counters = PathlistCounters::default();
    unsafe { count_paths((*rel).pathlist, false, &mut counters) };
    unsafe { count_paths((*rel).partial_pathlist, true, &mut counters) };

    OBSERVED.store(true, Ordering::Relaxed);
    RELID.store(unsafe { (*rel).relid }, Ordering::Relaxed);
    CONSIDER_PARALLEL.store(unsafe { (*rel).consider_parallel }, Ordering::Relaxed);
    REL_PARALLEL_WORKERS.store(unsafe { (*rel).rel_parallel_workers }, Ordering::Relaxed);
    EC_HNSW_INDEX_COUNT.store(ec_hnsw_index_count, Ordering::Relaxed);
    AMCANPARALLEL_SEEN.store(amcanparallel_seen, Ordering::Relaxed);
    PATH_COUNT.store(counters.path_count, Ordering::Relaxed);
    INDEX_PATH_COUNT.store(counters.index_path_count, Ordering::Relaxed);
    EC_HNSW_INDEX_PATH_COUNT.store(counters.ec_hnsw_index_path_count, Ordering::Relaxed);
    PARTIAL_PATH_COUNT.store(counters.partial_path_count, Ordering::Relaxed);
    PARTIAL_INDEX_PATH_COUNT.store(counters.partial_index_path_count, Ordering::Relaxed);
    PARTIAL_EC_HNSW_INDEX_PATH_COUNT
        .store(counters.partial_ec_hnsw_index_path_count, Ordering::Relaxed);
    store_path_summary(
        counters.best_plain_ec_hnsw,
        &BEST_PLAIN_STARTUP_COST,
        &BEST_PLAIN_TOTAL_COST,
        &BEST_PLAIN_PARALLEL_WORKERS,
        None,
        &BEST_PLAIN_PATHKEYS,
    );
    store_path_summary(
        counters.best_partial_ec_hnsw,
        &BEST_PARTIAL_STARTUP_COST,
        &BEST_PARTIAL_TOTAL_COST,
        &BEST_PARTIAL_PARALLEL_WORKERS,
        Some(&BEST_PARTIAL_PARALLEL_AWARE),
        &BEST_PARTIAL_PATHKEYS,
    );
}

fn previous_set_rel_pathlist_hook() -> pg_sys::set_rel_pathlist_hook_type {
    PREVIOUS_SET_REL_PATHLIST_HOOK
        .get()
        .copied()
        .unwrap_or(None)
}

unsafe extern "C-unwind" fn ecaz_set_rel_pathlist_hook(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rti: pg_sys::Index,
    rte: *mut pg_sys::RangeTblEntry,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if let Some(previous_hook) = previous_set_rel_pathlist_hook() {
                previous_hook(root, rel, rti, rte);
            }
            record_rel_pathlist(rel);
        })
    }
}

pub(crate) unsafe fn register_pg18_planner_hooks() {
    unsafe {
        if ECAZ_PLANNER_HOOK_REGISTERED.load(Ordering::Acquire) {
            return;
        }

        let _ = PREVIOUS_SET_REL_PATHLIST_HOOK.set(pg_sys::set_rel_pathlist_hook);
        pg_sys::set_rel_pathlist_hook = Some(ecaz_set_rel_pathlist_hook);
        ECAZ_PLANNER_HOOK_REGISTERED.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        choose_best_by_total, discounted_parallel_am_cost, parallel_cost_divisor, PathSummary,
    };

    #[test]
    fn choose_best_by_total_keeps_lowest_total_cost() {
        let mut current = Some(PathSummary {
            startup_cost: 10.0,
            total_cost: 20.0,
            parallel_workers: 0,
            parallel_aware: false,
            pathkeys: 1,
        });

        choose_best_by_total(
            &mut current,
            PathSummary {
                startup_cost: 15.0,
                total_cost: 25.0,
                parallel_workers: 4,
                parallel_aware: true,
                pathkeys: 2,
            },
        );
        assert_eq!(current.expect("best should be present").total_cost, 20.0);

        choose_best_by_total(
            &mut current,
            PathSummary {
                startup_cost: 30.0,
                total_cost: 5.0,
                parallel_workers: 2,
                parallel_aware: true,
                pathkeys: 3,
            },
        );
        let best = current.expect("best should be present");
        assert_eq!(best.total_cost, 5.0);
        assert_eq!(best.parallel_workers, 2);
        assert!(best.parallel_aware);
        assert_eq!(best.pathkeys, 3);
    }

    #[test]
    fn parallel_cost_divisor_matches_postgres_leader_formula() {
        assert_eq!(parallel_cost_divisor(0, true), None);
        assert_eq!(parallel_cost_divisor(1, true), Some(1.7));
        assert_eq!(parallel_cost_divisor(2, true), Some(2.4));
        assert_eq!(parallel_cost_divisor(3, true), Some(3.1));
        assert_eq!(parallel_cost_divisor(4, true), Some(4.0));
        assert_eq!(parallel_cost_divisor(4, false), Some(4.0));
    }

    #[test]
    fn discounted_parallel_am_cost_divides_index_work() {
        assert_eq!(discounted_parallel_am_cost(-1.0, 4, true), None);
        assert_eq!(discounted_parallel_am_cost(f64::INFINITY, 4, true), None);
        assert_eq!(discounted_parallel_am_cost(1000.0, 0, true), None);
        assert_eq!(discounted_parallel_am_cost(1000.0, 4, true), Some(250.0));

        let discounted = discounted_parallel_am_cost(1020.0, 2, true)
            .expect("two workers with leader participation should discount");
        assert!((discounted - 425.0).abs() < f64::EPSILON);
    }
}
