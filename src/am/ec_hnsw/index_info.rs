//! Typed wrappers around `pg_sys::BuildIndexInfo` for HNSW.
//!
//! `IndexInfoGuard` owns the `IndexInfo` allocation and `pfree`s it on drop.
//! Used where Rust controls the metadata lifetime (e.g. source-attribute
//! resolution).
//!
//! `IndexInfoView<'a>` borrows the `IndexInfo` from a surrounding PostgreSQL
//! memory context — no `pfree` on drop. Used where the PG context (e.g. a
//! parallel-build worker scope) reaps the allocation and an early Rust `pfree`
//! would double-free or release prematurely.
//!
//! Both wrap `NonNull<pg_sys::IndexInfo>` and share a common internal builder.

use std::marker::PhantomData;
use std::ptr::NonNull;

use pgrx::pg_sys;

fn build_inner(index_relation: pg_sys::Relation, label: &str) -> NonNull<pg_sys::IndexInfo> {
    let index_relation = NonNull::new(index_relation)
        .unwrap_or_else(|| pgrx::error!("ec_hnsw {label} needs a valid index relation"));
    // SAFETY: `index_relation` is a live PostgreSQL index relation; PostgreSQL
    // returns palloc'd IndexInfo metadata in the current memory context.
    let ptr = unsafe { pg_sys::BuildIndexInfo(index_relation.as_ptr()) };
    NonNull::new(ptr)
        .unwrap_or_else(|| pgrx::error!("ec_hnsw {label} could not build index metadata"))
}

pub(super) struct IndexInfoGuard {
    ptr: NonNull<pg_sys::IndexInfo>,
}

impl IndexInfoGuard {
    pub(super) fn build(index_relation: pg_sys::Relation, label: &str) -> Self {
        Self {
            ptr: build_inner(index_relation, label),
        }
    }

    pub(super) fn as_ptr(&self) -> *mut pg_sys::IndexInfo {
        self.ptr.as_ptr()
    }
}

impl Drop for IndexInfoGuard {
    fn drop(&mut self) {
        // SAFETY: `ptr` was allocated by PostgreSQL BuildIndexInfo and this
        // guard owns the matching pfree.
        unsafe { pg_sys::pfree(self.ptr.as_ptr().cast()) };
    }
}

pub(super) struct IndexInfoView<'scope> {
    ptr: NonNull<pg_sys::IndexInfo>,
    _scope: PhantomData<&'scope mut pg_sys::IndexInfo>,
}

impl<'scope> IndexInfoView<'scope> {
    pub(super) fn build_borrowed(index_relation: pg_sys::Relation, label: &str) -> Self {
        Self {
            ptr: build_inner(index_relation, label),
            _scope: PhantomData,
        }
    }

    pub(super) fn as_ptr(&self) -> *mut pg_sys::IndexInfo {
        self.ptr.as_ptr()
    }

    pub(super) fn set_concurrent(&mut self, is_concurrent: bool) {
        // SAFETY: `&mut self` enforces exclusive access; `ptr` is non-null by
        // construction; the mutation is bounded by the surrounding PG memory
        // context that owns the `IndexInfo` allocation.
        unsafe {
            self.ptr.as_mut().ii_Concurrent = is_concurrent;
        }
    }
}
