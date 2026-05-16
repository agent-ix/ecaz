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
pub(crate) fn register_kind() -> bool {
    // SAFETY: The linked C shim owns PG18 custom pgstat registration and takes
    // no Rust pointers. It returns false when the backend is not preload-ready.
    unsafe { ecaz_pg18_pgstat_register_kind() }
}

#[cfg(test)]
pub(crate) fn register_kind() -> bool {
    false
}

#[cfg(not(test))]
pub(crate) fn is_registered() -> bool {
    // SAFETY: The C shim only reads its registration flag and takes no Rust
    // pointers.
    unsafe { ecaz_pg18_pgstat_is_registered() }
}

#[cfg(test)]
pub(crate) fn is_registered() -> bool {
    false
}

#[cfg(not(test))]
pub(crate) fn record(delta: &TqStatsCounters) -> bool {
    // SAFETY: `delta` is a valid shared reference for the duration of this
    // synchronous C call; the shim copies the counter fields before returning.
    unsafe { ecaz_pg18_pgstat_record(delta as *const TqStatsCounters) }
}

#[cfg(test)]
pub(crate) fn record(_delta: &TqStatsCounters) -> bool {
    false
}

#[cfg(not(test))]
pub(crate) fn snapshot() -> Option<TqStatsCounters> {
    let mut snapshot = TqStatsCounters::default();
    // SAFETY: `snapshot` points to initialized writable Rust storage for the
    // duration of this synchronous C call.
    if unsafe { ecaz_pg18_pgstat_snapshot(&mut snapshot as *mut TqStatsCounters) } {
        Some(snapshot)
    } else {
        None
    }
}

#[cfg(test)]
pub(crate) fn snapshot() -> Option<TqStatsCounters> {
    None
}
