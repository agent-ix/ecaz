use crate::am::common::stats::TqStatsCounters;

#[cfg(not(test))]
#[link(name = "ecaz_pg18_pgstat_shim", kind = "static")]
unsafe extern "C" {
    fn ecaz_pg18_pgstat_anchor();
    fn ecaz_pg18_pgstat_register_kind() -> bool;
    fn ecaz_pg18_pgstat_is_registered() -> bool;
    fn ecaz_pg18_pgstat_record(delta: *const TqStatsCounters) -> bool;
    fn ecaz_pg18_pgstat_snapshot(out: *mut TqStatsCounters) -> bool;
}

#[cfg(not(test))]
#[used]
static PG18_PGSTAT_SHIM_ANCHOR: unsafe extern "C" fn() = ecaz_pg18_pgstat_anchor;

#[cfg(not(test))]
pub(crate) unsafe fn register_kind() -> bool {
    unsafe { ecaz_pg18_pgstat_register_kind() }
}

#[cfg(test)]
pub(crate) unsafe fn register_kind() -> bool {
    false
}

#[cfg(not(test))]
pub(crate) unsafe fn is_registered() -> bool {
    unsafe { ecaz_pg18_pgstat_is_registered() }
}

#[cfg(test)]
pub(crate) unsafe fn is_registered() -> bool {
    false
}

#[cfg(not(test))]
pub(crate) unsafe fn record(delta: &TqStatsCounters) -> bool {
    unsafe { ecaz_pg18_pgstat_record(delta as *const TqStatsCounters) }
}

#[cfg(test)]
pub(crate) unsafe fn record(_delta: &TqStatsCounters) -> bool {
    false
}

#[cfg(not(test))]
pub(crate) unsafe fn snapshot() -> Option<TqStatsCounters> {
    let mut snapshot = TqStatsCounters::default();
    if unsafe { ecaz_pg18_pgstat_snapshot(&mut snapshot as *mut TqStatsCounters) } {
        Some(snapshot)
    } else {
        None
    }
}

#[cfg(test)]
pub(crate) unsafe fn snapshot() -> Option<TqStatsCounters> {
    None
}
