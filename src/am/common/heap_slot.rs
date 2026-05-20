use std::{marker::PhantomData, ptr::NonNull};

use pgrx::{itemptr::item_pointer_set_all, pg_sys};

use crate::storage::page::ItemPointer;

pub(crate) struct TupleSlotReader<'slot> {
    slot: NonNull<pg_sys::TupleTableSlot>,
    am_name: &'static str,
    _slot: PhantomData<&'slot mut pg_sys::TupleTableSlot>,
}

impl<'slot> TupleSlotReader<'slot> {
    pub(crate) unsafe fn from_raw_slot(
        slot: *mut pg_sys::TupleTableSlot,
        am_name: &'static str,
    ) -> Result<Self, String> {
        let slot = NonNull::new(slot)
            .ok_or_else(|| format!("{am_name} slot reader received a null tuple slot"))?;
        Ok(Self {
            slot,
            am_name,
            _slot: PhantomData,
        })
    }

    pub(crate) fn clear(&mut self) {
        // SAFETY: construction requires a live TupleTableSlot owned by the
        // caller for this callback scope.
        unsafe { pg_sys::ExecClearTuple(self.slot.as_ptr()) };
    }

    pub(crate) fn required_datum(
        &mut self,
        attnum: i32,
        label: &str,
    ) -> Result<pg_sys::Datum, String> {
        let attr_index = usize::try_from(attnum - 1)
            .map_err(|_| format!("{} heap attribute number must be positive", self.am_name))?;

        // SAFETY: construction requires a live slot and attnum names an
        // indexed/source attribute resolved from relation metadata.
        if unsafe { self.slot.as_ref().tts_nvalid } < attnum as i16 {
            // SAFETY: materializes attributes through attnum for the same live
            // slot owned by this reader.
            unsafe { pg_sys::slot_getsomeattrs_int(self.slot.as_ptr(), attnum) };
        }
        // SAFETY: the slot has materialized at least attnum attributes, so the
        // null and datum arrays contain attr_index.
        if unsafe { *self.slot.as_ref().tts_isnull.add(attr_index) } {
            return Err(format!("{} does not support NULL {label}", self.am_name));
        }
        // SAFETY: the materialized attribute is non-null.
        Ok(unsafe { *self.slot.as_ref().tts_values.add(attr_index) })
    }
}

pub(crate) struct HeapSlotReader<'slot> {
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: TupleSlotReader<'slot>,
}

impl<'slot> HeapSlotReader<'slot> {
    pub(crate) unsafe fn from_raw(
        heap_relation: pg_sys::Relation,
        snapshot: pg_sys::Snapshot,
        slot: *mut pg_sys::TupleTableSlot,
        am_name: &'static str,
    ) -> Result<Self, String> {
        if heap_relation.is_null() {
            return Err(format!(
                "{am_name} heap slot reader received a null heap relation"
            ));
        }
        if snapshot.is_null() {
            return Err(format!(
                "{am_name} heap slot reader received a null snapshot"
            ));
        }
        let slot = unsafe { TupleSlotReader::from_raw_slot(slot, am_name)? };
        Ok(Self {
            heap_relation,
            snapshot,
            slot,
        })
    }

    pub(crate) fn clear(&mut self) {
        self.slot.clear();
    }

    pub(crate) fn fetch_row_version(&mut self, heap_tid: ItemPointer) -> Result<bool, String> {
        let mut tid = pg_sys::ItemPointerData::default();
        item_pointer_set_all(&mut tid, heap_tid.block_number, heap_tid.offset_number);
        self.clear();
        // SAFETY: construction binds a live heap relation, snapshot, and tuple
        // slot. The TID was initialized from the index candidate being fetched.
        Ok(unsafe {
            pg_sys::table_tuple_fetch_row_version(
                self.heap_relation,
                &mut tid,
                self.snapshot,
                self.slot.slot.as_ptr(),
            )
        })
    }

    pub(crate) fn required_datum(
        &mut self,
        attnum: i32,
        label: &str,
    ) -> Result<pg_sys::Datum, String> {
        self.slot.required_datum(attnum, label)
    }
}
