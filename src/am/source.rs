use std::{ffi::c_int, ptr};

use pgrx::{itemptr::item_pointer_set_all, pg_sys, PgTupleDesc};

use super::page;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SourceTypePolicy {
    RealArrayOnly,
    RealArrayOrBytea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SourceAttribute {
    pub(super) attnum: i32,
    pub(super) type_oid: pg_sys::Oid,
}

pub(super) fn average_source_representatives(
    existing: &mut [f32],
    existing_count: usize,
    incoming: &[f32],
    incoming_count: usize,
) {
    assert_eq!(existing.len(), incoming.len());
    assert!(existing_count > 0);
    assert!(incoming_count > 0);

    let total_count = existing_count + incoming_count;
    for (existing_value, incoming_value) in existing.iter_mut().zip(incoming.iter()) {
        *existing_value = ((*existing_value * existing_count as f32)
            + (*incoming_value * incoming_count as f32))
            / total_count as f32;
    }
}

pub(super) unsafe fn resolve_source_attnum(
    heap_relation: pg_sys::Relation,
    source_column: &str,
    source_label: &str,
) -> i32 {
    let source_column = std::ffi::CString::new(source_column)
        .unwrap_or_else(|_| pgrx::error!("tqhnsw {source_label} contains an invalid NUL byte"));
    let attnum = unsafe { pg_sys::get_attnum((*heap_relation).rd_id, source_column.as_ptr()) };
    let attnum = i32::from(attnum);
    if attnum <= 0 {
        pgrx::error!(
            "tqhnsw {source_label} \"{}\" does not name a user column on the heap relation",
            source_column.to_string_lossy()
        );
    }
    attnum
}

pub(super) unsafe fn resolve_source_attribute(
    heap_relation: pg_sys::Relation,
    source_column: &str,
    source_label: &str,
    type_policy: SourceTypePolicy,
) -> SourceAttribute {
    let source_attnum =
        unsafe { resolve_source_attnum(heap_relation, source_column, source_label) };
    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(source_attnum as usize - 1)
        .expect("resolved source attribute should exist");
    if att.attisdropped {
        pgrx::error!("tqhnsw {source_label} \"{source_column}\" references a dropped column");
    }

    match type_policy {
        SourceTypePolicy::RealArrayOnly => {
            if att.atttypid != pg_sys::FLOAT4ARRAYOID {
                pgrx::error!(
                    "tqhnsw {source_label} \"{source_column}\" must be real[], got type oid {}",
                    u32::from(att.atttypid)
                );
            }
        }
        SourceTypePolicy::RealArrayOrBytea => {
            if att.atttypid != pg_sys::FLOAT4ARRAYOID && att.atttypid != pg_sys::BYTEAOID {
                pgrx::error!(
                    "tqhnsw {source_label} \"{source_column}\" must be real[] or bytea, got type oid {}",
                    u32::from(att.atttypid)
                );
            }
        }
    }

    SourceAttribute {
        attnum: source_attnum,
        type_oid: att.atttypid,
    }
}

pub(super) unsafe fn allocate_heap_slot(
    heap_relation: pg_sys::Relation,
    failure_label: &str,
) -> *mut pg_sys::TupleTableSlot {
    let slot = unsafe {
        pg_sys::MakeSingleTupleTableSlot(
            (*heap_relation).rd_att,
            pg_sys::table_slot_callbacks(heap_relation),
        )
    };
    if slot.is_null() {
        pgrx::error!("{failure_label}");
    }
    slot
}

pub(super) unsafe fn fetch_heap_row_version(
    heap_relation: pg_sys::Relation,
    heap_tid: page::ItemPointer,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    label: &str,
) {
    let mut tid = pg_sys::ItemPointerData::default();
    item_pointer_set_all(&mut tid, heap_tid.block_number, heap_tid.offset_number);
    unsafe { pg_sys::ExecClearTuple(slot) };
    let fetched =
        unsafe { pg_sys::table_tuple_fetch_row_version(heap_relation, &mut tid, snapshot, slot) };
    if !fetched {
        pgrx::error!(
            "tqhnsw {label} could not fetch heap tuple at ({},{})",
            heap_tid.block_number,
            heap_tid.offset_number
        );
    }
}

pub(super) unsafe fn required_slot_datum(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    label: &str,
) -> pg_sys::Datum {
    if unsafe { (*slot).tts_nvalid } < attnum as i16 {
        unsafe { pg_sys::slot_getsomeattrs_int(slot, attnum) };
    }
    let attr_index = usize::try_from(attnum - 1).expect("attribute number should be positive");
    if unsafe { *(*slot).tts_isnull.add(attr_index) } {
        pgrx::error!("tqhnsw does not support NULL {label}");
    }
    unsafe { *(*slot).tts_values.add(attr_index) }
}

pub(super) struct FlatFloat4ArrayRef {
    array_ptr: *mut pg_sys::ArrayType,
    owned: bool,
    data_ptr: *const f32,
    len: usize,
}

impl FlatFloat4ArrayRef {
    pub(super) unsafe fn from_datum(datum: pg_sys::Datum, label: &str) -> Self {
        if datum.is_null() {
            pgrx::error!("tqhnsw does not support NULL {label}");
        }

        let original = datum
            .cast_mut_ptr::<std::ffi::c_void>()
            .cast::<pg_sys::varlena>();
        let varlena = unsafe { pg_sys::pg_detoast_datum(original.cast()) };
        if varlena.is_null() {
            pgrx::error!("tqhnsw could not detoast {label}");
        }
        let array_ptr = varlena.cast::<pg_sys::ArrayType>();
        let owned = !ptr::eq(varlena, original);

        let ndim = match usize::try_from(unsafe { (*array_ptr).ndim }) {
            Ok(value) => value,
            Err(_) => pgrx::error!("tqhnsw {label} must be a one-dimensional real[]"),
        };
        if ndim != 1 {
            pgrx::error!("tqhnsw {label} must be a one-dimensional real[]");
        }
        if unsafe { (*array_ptr).elemtype } != pg_sys::FLOAT4OID {
            pgrx::error!("tqhnsw {label} must be a real[]");
        }
        if unsafe { pg_sys::array_contains_nulls(array_ptr) } {
            pgrx::error!("tqhnsw {label} arrays must not contain NULL elements");
        }

        let dims_ptr = unsafe { flat_array_dims_ptr(array_ptr) };
        let len = usize::try_from(unsafe { pg_sys::ArrayGetNItems((*array_ptr).ndim, dims_ptr) })
            .expect("flat float4 array length should fit in usize");
        let data_ptr = unsafe {
            array_ptr
                .cast::<u8>()
                .add(flat_array_data_offset(array_ptr, ndim))
                .cast::<f32>()
        };
        if (data_ptr as usize) % std::mem::align_of::<f32>() != 0 {
            pgrx::error!("tqhnsw {label} data pointer is not aligned for float4 access");
        }

        Self {
            array_ptr,
            owned,
            data_ptr,
            len,
        }
    }

    pub(super) fn as_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.len) }
    }
}

impl Drop for FlatFloat4ArrayRef {
    fn drop(&mut self) {
        if self.owned {
            unsafe { pg_sys::pfree(self.array_ptr.cast()) };
        }
    }
}

pub(super) struct FlatFloat4ByteaRef {
    varlena_ptr: *mut pg_sys::varlena,
    owned: bool,
    data_ptr: *const f32,
    len: usize,
}

impl FlatFloat4ByteaRef {
    pub(super) unsafe fn from_datum(datum: pg_sys::Datum, label: &str) -> Self {
        if datum.is_null() {
            pgrx::error!("tqhnsw does not support NULL {label}");
        }

        let original = datum
            .cast_mut_ptr::<std::ffi::c_void>()
            .cast::<pg_sys::varlena>();
        let varlena = unsafe { pg_sys::pg_detoast_datum(original.cast()) };
        if varlena.is_null() {
            pgrx::error!("tqhnsw could not detoast {label}");
        }
        let owned = !ptr::eq(varlena, original);
        let bytes = unsafe { pgrx::varlena::varlena_to_byte_slice(varlena) };
        if bytes.len() % std::mem::size_of::<f32>() != 0 {
            pgrx::error!("tqhnsw {label} bytea payload length must be a multiple of 4 bytes");
        }
        let (prefix, body, suffix) = unsafe { bytes.align_to::<f32>() };
        if !prefix.is_empty() || !suffix.is_empty() {
            pgrx::error!("tqhnsw {label} bytea payload is not aligned for float4 access");
        }

        Self {
            varlena_ptr: varlena,
            owned,
            data_ptr: body.as_ptr(),
            len: body.len(),
        }
    }

    pub(super) fn as_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.len) }
    }
}

impl Drop for FlatFloat4ByteaRef {
    fn drop(&mut self) {
        if self.owned {
            unsafe { pg_sys::pfree(self.varlena_ptr.cast()) };
        }
    }
}

pub(super) enum FlatFloat4SourceRef {
    Array(FlatFloat4ArrayRef),
    Bytea(FlatFloat4ByteaRef),
}

impl FlatFloat4SourceRef {
    pub(super) unsafe fn from_datum(
        datum: pg_sys::Datum,
        type_oid: pg_sys::Oid,
        label: &str,
    ) -> Self {
        match type_oid {
            pg_sys::FLOAT4ARRAYOID => {
                Self::Array(unsafe { FlatFloat4ArrayRef::from_datum(datum, label) })
            }
            pg_sys::BYTEAOID => {
                Self::Bytea(unsafe { FlatFloat4ByteaRef::from_datum(datum, label) })
            }
            _ => pgrx::error!(
                "tqhnsw {label} must be real[] or bytea, got type oid {}",
                u32::from(type_oid)
            ),
        }
    }

    pub(super) fn as_slice(&self) -> &[f32] {
        match self {
            Self::Array(array) => array.as_slice(),
            Self::Bytea(bytea) => bytea.as_slice(),
        }
    }
}

pub(super) unsafe fn load_source_from_heap_row(
    heap_relation: pg_sys::Relation,
    heap_tid: page::ItemPointer,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attribute: SourceAttribute,
    label: &str,
) -> FlatFloat4SourceRef {
    unsafe { fetch_heap_row_version(heap_relation, heap_tid, snapshot, slot, label) };
    let source_datum = unsafe { required_slot_datum(slot, source_attribute.attnum, label) };
    unsafe { FlatFloat4SourceRef::from_datum(source_datum, source_attribute.type_oid, label) }
}

pub(super) fn negative_inner_product(query: &[f32], source: &[f32]) -> f32 {
    if query.len() != source.len() {
        pgrx::error!(
            "tqhnsw source vector dimension mismatch: left dim {}, right dim {}",
            query.len(),
            source.len()
        );
    }
    -query
        .iter()
        .zip(source)
        .map(|(left, right)| left * right)
        .sum::<f32>()
}

unsafe fn flat_array_dims_ptr(array_ptr: *const pg_sys::ArrayType) -> *const c_int {
    unsafe {
        array_ptr
            .cast::<u8>()
            .add(std::mem::size_of::<pg_sys::ArrayType>())
            .cast::<c_int>()
    }
}

fn maxaligned_size(len: usize) -> usize {
    let align =
        usize::try_from(pg_sys::MAXIMUM_ALIGNOF).expect("MAXIMUM_ALIGNOF should fit in usize");
    (len + align - 1) & !(align - 1)
}

unsafe fn flat_array_data_offset(array_ptr: *const pg_sys::ArrayType, ndim: usize) -> usize {
    let dataoffset = unsafe { (*array_ptr).dataoffset };
    if dataoffset != 0 {
        usize::try_from(dataoffset).expect("flat float4 array dataoffset should fit in usize")
    } else {
        maxaligned_size(
            std::mem::size_of::<pg_sys::ArrayType>() + (2 * ndim * std::mem::size_of::<c_int>()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_source_representatives_tracks_weighted_mean() {
        let mut representative = vec![1.0_f32, 0.0];
        average_source_representatives(&mut representative, 1, &[0.0, 1.0], 1);
        average_source_representatives(&mut representative, 2, &[1.0, 1.0], 2);

        assert_eq!(representative, vec![0.75_f32, 0.75_f32]);
    }

    #[test]
    fn negative_inner_product_matches_expected_sign() {
        assert_eq!(
            negative_inner_product(&[1.0_f32, -2.0, 0.5], &[0.5_f32, 2.0, -1.0]),
            4.0_f32
        );
    }
}
