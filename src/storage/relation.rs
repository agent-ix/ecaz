//! PostgreSQL relation descriptor helpers.

use std::ffi::{c_char, CStr};
use std::ptr::NonNull;

use pgrx::{pg_sys, PgTupleDesc};

pub(crate) type RelationHandle = NonNull<pg_sys::RelationData>;

pub(crate) fn main_fork_block_count_handle(relation: RelationHandle) -> pg_sys::BlockNumber {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; this
    // only asks PostgreSQL for the current MAIN fork block count by value.
    unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(relation.as_ptr(), pg_sys::ForkNumber::MAIN_FORKNUM)
    }
}

pub(crate) fn relation_oid_handle(relation: RelationHandle) -> pg_sys::Oid {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_id
    // is copied by value and no relation-owned memory is retained.
    unsafe { (*relation.as_ptr()).rd_id }
}

pub(crate) fn relation_reltuples_handle(relation: RelationHandle) -> f64 {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and reltuples is copied by value.
    unsafe { (*(*relation.as_ptr()).rd_rel).reltuples as f64 }
}

pub(crate) fn relation_tablespace_handle(relation: RelationHandle) -> pg_sys::Oid {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and reltablespace is copied by value.
    unsafe { (*(*relation.as_ptr()).rd_rel).reltablespace }
}

pub(crate) fn relation_name_handle(relation: RelationHandle) -> String {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and relname is PostgreSQL's fixed C string.
    unsafe { CStr::from_ptr((*(*relation.as_ptr()).rd_rel).relname.data.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

pub(crate) fn relation_kind_handle(relation: RelationHandle) -> c_char {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and relkind is copied by value.
    unsafe { (*(*relation.as_ptr()).rd_rel).relkind }
}

pub(crate) fn relation_am_oid_handle(relation: RelationHandle) -> pg_sys::Oid {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and relam is copied by value.
    unsafe { (*(*relation.as_ptr()).rd_rel).relam }
}

pub(crate) fn relation_namespace_owner_persistence_handle(
    relation: RelationHandle,
) -> (pg_sys::Oid, pg_sys::Oid, c_char) {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and fields are copied by value.
    unsafe {
        let rd_rel = &*(*relation.as_ptr()).rd_rel;
        (rd_rel.relnamespace, rd_rel.relowner, rd_rel.relpersistence)
    }
}

pub(crate) fn relation_tuple_desc_copy_handle(relation: RelationHandle) -> PgTupleDesc<'static> {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_att
    // points at PostgreSQL-owned tuple descriptor metadata and PgTupleDesc makes
    // a CurrentMemoryContext copy that owns its lifetime.
    unsafe { PgTupleDesc::from_pg_copy((*relation.as_ptr()).rd_att) }
}

pub(crate) fn relation_raw_tuple_desc_copy_handle(relation: RelationHandle) -> pg_sys::TupleDesc {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_att
    // points at PostgreSQL-owned tuple descriptor metadata and PostgreSQL
    // returns a freshly allocated copy for catalog creation callers.
    unsafe { pg_sys::CreateTupleDescCopy((*relation.as_ptr()).rd_att) }
}

pub(crate) fn relation_options_handle(relation: RelationHandle) -> *mut pg_sys::varlena {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; the
    // reloptions pointer is copied by value and remains relation-owned.
    unsafe { (*relation.as_ptr()).rd_options }
}

pub(crate) fn index_heap_relation_oid_handle(index_relation: RelationHandle) -> pg_sys::Oid {
    index_heap_relation_oid_from_index_oid(relation_oid_handle(index_relation))
}

pub(crate) fn index_heap_relation_oid_from_index_oid(index_oid: pg_sys::Oid) -> pg_sys::Oid {
    // SAFETY: asks PostgreSQL catalog metadata for the heap relation linked to
    // the copied index OID and returns that OID by value.
    unsafe { pg_sys::IndexGetRelation(index_oid, false) }
}
