use pgrx::pg_sys;

pub(crate) struct TupleTableSlotGuard {
    slot: *mut pg_sys::TupleTableSlot,
}

impl TupleTableSlotGuard {
    pub(crate) fn create(relation: pg_sys::Relation) -> Option<Self> {
        // SAFETY: `relation` is owned by a live relation guard in the caller;
        // this guard owns the returned slot.
        let slot = unsafe { pg_sys::table_slot_create(relation, std::ptr::null_mut()) };
        if slot.is_null() {
            return None;
        }
        Some(Self { slot })
    }

    pub(crate) fn single_for_heap(relation: pg_sys::Relation) -> Option<Self> {
        // SAFETY: `relation` is an open heap relation. PostgreSQL owns the
        // returned slot until `ExecDropSingleTupleTableSlot`.
        let slot = unsafe {
            pg_sys::MakeSingleTupleTableSlot(
                (*relation).rd_att,
                pg_sys::table_slot_callbacks(relation),
            )
        };
        if slot.is_null() {
            return None;
        }
        Some(Self { slot })
    }

    pub(crate) fn as_ptr(&self) -> *mut pg_sys::TupleTableSlot {
        self.slot
    }
}

impl Drop for TupleTableSlotGuard {
    fn drop(&mut self) {
        // SAFETY: `slot` was returned by one of this guard's constructors;
        // this guard owns the matching drop.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::ExecDropSingleTupleTableSlot(self.slot) };
    }
}
