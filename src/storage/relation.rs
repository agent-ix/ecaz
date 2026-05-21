//! PostgreSQL relation descriptor helpers.

use pgrx::pg_sys;

pub(crate) fn main_fork_block_count(relation: pg_sys::Relation) -> pg_sys::BlockNumber {
    if relation.is_null() {
        pgrx::error!("main fork block count needs a valid relation");
    }
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; this
    // only asks PostgreSQL for the current MAIN fork block count by value.
    unsafe { pg_sys::RelationGetNumberOfBlocksInFork(relation, pg_sys::ForkNumber::MAIN_FORKNUM) }
}
