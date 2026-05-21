use std::marker::PhantomData;

use pgrx::pg_sys;

use super::relation_guard::HeapRelationGuard;

#[derive(Debug)]
pub(crate) struct TupleTableSlotGuard<'rel> {
    slot: *mut pg_sys::TupleTableSlot,
    _relation: PhantomData<&'rel pg_sys::RelationData>,
}

impl<'rel> TupleTableSlotGuard<'rel> {
    pub(crate) fn create_for_heap_guard(relation: &'rel HeapRelationGuard) -> Option<Self> {
        // SAFETY: the heap relation guard borrow bounds the returned slot.
        let slot = unsafe { pg_sys::table_slot_create(relation.as_ptr(), std::ptr::null_mut()) };
        if slot.is_null() {
            return None;
        }
        Some(Self {
            slot,
            _relation: PhantomData,
        })
    }

    pub(crate) fn single_for_heap_guard(relation: &'rel HeapRelationGuard) -> Option<Self> {
        // SAFETY: the heap relation guard borrow bounds the returned slot.
        let slot = unsafe {
            pg_sys::MakeSingleTupleTableSlot(
                (*relation.as_ptr()).rd_att,
                pg_sys::table_slot_callbacks(relation.as_ptr()),
            )
        };
        if slot.is_null() {
            return None;
        }
        Some(Self {
            slot,
            _relation: PhantomData,
        })
    }
}

impl TupleTableSlotGuard<'static> {
    pub(crate) unsafe fn single_for_heap(relation: pg_sys::Relation) -> Option<Self> {
        // SAFETY: callers that only have a raw relation pointer must uphold
        // the relation liveness contract for the returned raw-boundary guard.
        let slot = unsafe {
            pg_sys::MakeSingleTupleTableSlot(
                (*relation).rd_att,
                pg_sys::table_slot_callbacks(relation),
            )
        };
        if slot.is_null() {
            return None;
        }
        Some(Self {
            slot,
            _relation: PhantomData,
        })
    }
}

impl TupleTableSlotGuard<'_> {
    pub(crate) fn as_ptr(&self) -> *mut pg_sys::TupleTableSlot {
        self.slot
    }
}

impl Drop for TupleTableSlotGuard<'_> {
    fn drop(&mut self) {
        // SAFETY: `slot` was returned by one of this guard's constructors;
        // this guard owns the matching drop.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::ExecDropSingleTupleTableSlot(self.slot) };
    }
}
