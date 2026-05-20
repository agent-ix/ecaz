use pgrx::{itemptr::item_pointer_set_all, pg_sys};

use crate::storage::page::ItemPointer;

pub(crate) unsafe fn clear_tuple_slot(slot: *mut pg_sys::TupleTableSlot) {
    // SAFETY: caller owns a live TupleTableSlot for the current callback and
    // permits PostgreSQL to clear it before reuse.
    unsafe { pg_sys::ExecClearTuple(slot) };
}

pub(crate) unsafe fn fetch_heap_row_version(
    heap_relation: pg_sys::Relation,
    heap_tid: ItemPointer,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    am_name: &str,
) -> Result<bool, String> {
    if slot.is_null() {
        return Err(format!("{am_name} heap fetch received a null tuple slot"));
    }

    let mut tid = pg_sys::ItemPointerData::default();
    item_pointer_set_all(&mut tid, heap_tid.block_number, heap_tid.offset_number);
    // SAFETY: caller owns a live slot; clearing before fetch matches the
    // PostgreSQL slot reuse contract for table_tuple_fetch_row_version.
    unsafe { clear_tuple_slot(slot) };
    // SAFETY: heap_relation, snapshot, and slot are live for the callback, and
    // tid was initialized from the index tuple's heap TID.
    Ok(unsafe { pg_sys::table_tuple_fetch_row_version(heap_relation, &mut tid, snapshot, slot) })
}

pub(crate) unsafe fn required_slot_datum(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    am_name: &str,
    label: &str,
) -> Result<pg_sys::Datum, String> {
    if slot.is_null() {
        return Err(format!(
            "{am_name} slot datum lookup received a null tuple slot"
        ));
    }
    let attr_index = usize::try_from(attnum - 1)
        .map_err(|_| format!("{am_name} heap attribute number must be positive"))?;

    // SAFETY: slot is live and attnum names the one-based indexed/source
    // attribute resolved from relation metadata.
    if unsafe { (*slot).tts_nvalid } < attnum as i16 {
        // SAFETY: materializes attributes through attnum for the same live slot.
        unsafe { pg_sys::slot_getsomeattrs_int(slot, attnum) };
    }
    // SAFETY: the slot has materialized at least attnum attributes, so the
    // null and datum arrays contain attr_index.
    if unsafe { *(*slot).tts_isnull.add(attr_index) } {
        return Err(format!("{am_name} does not support NULL {label}"));
    }
    // SAFETY: the materialized attribute is non-null.
    Ok(unsafe { *(*slot).tts_values.add(attr_index) })
}
