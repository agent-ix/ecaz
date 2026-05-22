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

pub(crate) unsafe fn main_fork_block_count(relation: pg_sys::Relation) -> pg_sys::BlockNumber {
    let relation = NonNull::new(relation)
        .unwrap_or_else(|| pgrx::error!("main fork block count needs a valid relation"));
    main_fork_block_count_handle(relation)
}

pub(crate) unsafe fn relation_oid(relation: pg_sys::Relation) -> pg_sys::Oid {
    let relation = NonNull::new(relation)
        .unwrap_or_else(|| pgrx::error!("relation OID read needs a valid relation"));
    relation_oid_handle(relation)
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

pub(crate) unsafe fn relation_tablespace(relation: pg_sys::Relation) -> pg_sys::Oid {
    let relation = NonNull::new(relation)
        .unwrap_or_else(|| pgrx::error!("relation tablespace read needs a valid relation"));
    relation_tablespace_handle(relation)
}

pub(crate) fn relation_tablespace_handle(relation: RelationHandle) -> pg_sys::Oid {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and reltablespace is copied by value.
    unsafe { (*(*relation.as_ptr()).rd_rel).reltablespace }
}

pub(crate) unsafe fn relation_name(relation: pg_sys::Relation) -> String {
    if relation.is_null() {
        pgrx::error!("relation name read needs a valid relation");
    }
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and relname is PostgreSQL's fixed C string.
    unsafe { CStr::from_ptr((*(*relation).rd_rel).relname.data.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

pub(crate) unsafe fn relation_kind(relation: pg_sys::Relation) -> c_char {
    if relation.is_null() {
        pgrx::error!("relation kind read needs a valid relation");
    }
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and relkind is copied by value.
    unsafe { (*(*relation).rd_rel).relkind }
}

pub(crate) unsafe fn relation_am_oid(relation: pg_sys::Relation) -> pg_sys::Oid {
    let relation = NonNull::new(relation)
        .unwrap_or_else(|| pgrx::error!("relation access method read needs a valid relation"));
    relation_am_oid_handle(relation)
}

pub(crate) fn relation_am_oid_handle(relation: RelationHandle) -> pg_sys::Oid {
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and relam is copied by value.
    unsafe { (*(*relation.as_ptr()).rd_rel).relam }
}

pub(crate) unsafe fn relation_namespace_owner_persistence(
    relation: pg_sys::Relation,
) -> (pg_sys::Oid, pg_sys::Oid, c_char) {
    if relation.is_null() {
        pgrx::error!("relation catalog field read needs a valid relation");
    }
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_rel
    // belongs to that descriptor and fields are copied by value.
    unsafe {
        let rd_rel = &*(*relation).rd_rel;
        (rd_rel.relnamespace, rd_rel.relowner, rd_rel.relpersistence)
    }
}

pub(crate) unsafe fn relation_tuple_desc_copy(relation: pg_sys::Relation) -> PgTupleDesc<'static> {
    if relation.is_null() {
        pgrx::error!("relation tuple descriptor copy needs a valid relation");
    }
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_att
    // points at PostgreSQL-owned tuple descriptor metadata and PgTupleDesc makes
    // a CurrentMemoryContext copy that owns its lifetime.
    unsafe { PgTupleDesc::from_pg_copy((*relation).rd_att) }
}

pub(crate) unsafe fn relation_raw_tuple_desc_copy(relation: pg_sys::Relation) -> pg_sys::TupleDesc {
    if relation.is_null() {
        pgrx::error!("raw relation tuple descriptor copy needs a valid relation");
    }
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; rd_att
    // points at PostgreSQL-owned tuple descriptor metadata and PostgreSQL
    // returns a freshly allocated copy for catalog creation callers.
    unsafe { pg_sys::CreateTupleDescCopy((*relation).rd_att) }
}

pub(crate) unsafe fn relation_options(relation: pg_sys::Relation) -> *mut pg_sys::varlena {
    if relation.is_null() {
        pgrx::error!("relation options read needs a valid relation");
    }
    // SAFETY: callers pass a live opened PostgreSQL relation descriptor; the
    // reloptions pointer is copied by value and remains relation-owned.
    unsafe { (*relation).rd_options }
}

pub(crate) unsafe fn index_heap_relation_oid(index_relation: pg_sys::Relation) -> pg_sys::Oid {
    let index_relation = NonNull::new(index_relation)
        .unwrap_or_else(|| pgrx::error!("index heap relation OID read needs a valid relation"));
    index_heap_relation_oid_handle(index_relation)
}

pub(crate) fn index_heap_relation_oid_handle(index_relation: RelationHandle) -> pg_sys::Oid {
    index_heap_relation_oid_from_index_oid(relation_oid_handle(index_relation))
}

pub(crate) fn index_heap_relation_oid_from_index_oid(index_oid: pg_sys::Oid) -> pg_sys::Oid {
    // SAFETY: asks PostgreSQL catalog metadata for the heap relation linked to
    // the copied index OID and returns that OID by value.
    unsafe { pg_sys::IndexGetRelation(index_oid, false) }
}
