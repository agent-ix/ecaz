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

pub(crate) fn relation_oid(relation: pg_sys::Relation) -> pg_sys::Oid {
    if relation.is_null() {
        pgrx::error!("relation OID read needs a valid relation");
    }
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_id
    // is copied by value and no relation-owned memory is retained.
    unsafe { (*relation).rd_id }
}

pub(crate) fn index_heap_relation_oid(index_relation: pg_sys::Relation) -> pg_sys::Oid {
    index_heap_relation_oid_from_index_oid(relation_oid(index_relation))
}

pub(crate) fn index_heap_relation_oid_from_index_oid(index_oid: pg_sys::Oid) -> pg_sys::Oid {
    // SAFETY: asks PostgreSQL catalog metadata for the heap relation linked to
    // the copied index OID and returns that OID by value.
    unsafe { pg_sys::IndexGetRelation(index_oid, false) }
}
