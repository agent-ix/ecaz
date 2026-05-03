use std::{
    ffi::{c_int, CStr},
    ptr,
};

use pgrx::{itemptr::item_pointer_set_all, pg_sys, PgTupleDesc};

use super::page;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::{float32x4_t, vaddq_f32, vdupq_n_f32, vfmaq_f32, vld1q_f32, vst1q_f32};
#[cfg(target_arch = "x86")]
use std::arch::x86::{
    __m256, _mm256_add_ps, _mm256_fmadd_ps, _mm256_loadu_ps, _mm256_setzero_ps, _mm256_storeu_ps,
};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256, _mm256_add_ps, _mm256_fmadd_ps, _mm256_loadu_ps, _mm256_setzero_ps, _mm256_storeu_ps,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceTypePolicy {
    BuildSource,
    RerankSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub(crate) enum SourceDatumKind {
    #[default]
    Unknown = 0,
    RealArray = 1,
    Bytea = 2,
    Ecvector = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IndexedVectorKind {
    Ecvector,
    Tqvector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceAttribute {
    pub(crate) attnum: i32,
    pub(crate) kind: SourceDatumKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IndexedVectorAttribute {
    pub(crate) attnum: i32,
    pub(crate) kind: IndexedVectorKind,
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

pub(crate) fn inner_product(left: &[f32], right: &[f32]) -> f32 {
    debug_assert_eq!(left.len(), right.len());
    let len = left.len().min(right.len());
    let left = &left[..len];
    let right = &right[..len];

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma")
        {
            return unsafe { inner_product_avx2_fma(left, right) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return unsafe { inner_product_neon(left, right) };
        }
    }

    inner_product_scalar(left, right)
}

fn inner_product_scalar(left: &[f32], right: &[f32]) -> f32 {
    let mut sum = 0.0_f32;
    let chunk_len = left.len() / 4 * 4;
    for (left, right) in left[..chunk_len]
        .chunks_exact(4)
        .zip(right[..chunk_len].chunks_exact(4))
    {
        sum += left[0] * right[0];
        sum += left[1] * right[1];
        sum += left[2] * right[2];
        sum += left[3] * right[3];
    }
    for (left, right) in left[chunk_len..].iter().zip(right[chunk_len..].iter()) {
        sum += left * right;
    }
    sum
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn inner_product_avx2_fma(left: &[f32], right: &[f32]) -> f32 {
    let mut acc0: __m256 = _mm256_setzero_ps();
    let mut acc1: __m256 = _mm256_setzero_ps();
    let mut acc2: __m256 = _mm256_setzero_ps();
    let mut acc3: __m256 = _mm256_setzero_ps();
    let mut offset = 0_usize;
    while offset + 32 <= left.len() {
        let l0 = unsafe { _mm256_loadu_ps(left.as_ptr().add(offset)) };
        let r0 = unsafe { _mm256_loadu_ps(right.as_ptr().add(offset)) };
        let l1 = unsafe { _mm256_loadu_ps(left.as_ptr().add(offset + 8)) };
        let r1 = unsafe { _mm256_loadu_ps(right.as_ptr().add(offset + 8)) };
        let l2 = unsafe { _mm256_loadu_ps(left.as_ptr().add(offset + 16)) };
        let r2 = unsafe { _mm256_loadu_ps(right.as_ptr().add(offset + 16)) };
        let l3 = unsafe { _mm256_loadu_ps(left.as_ptr().add(offset + 24)) };
        let r3 = unsafe { _mm256_loadu_ps(right.as_ptr().add(offset + 24)) };
        acc0 = _mm256_fmadd_ps(l0, r0, acc0);
        acc1 = _mm256_fmadd_ps(l1, r1, acc1);
        acc2 = _mm256_fmadd_ps(l2, r2, acc2);
        acc3 = _mm256_fmadd_ps(l3, r3, acc3);
        offset += 32;
    }
    while offset + 8 <= left.len() {
        let l = unsafe { _mm256_loadu_ps(left.as_ptr().add(offset)) };
        let r = unsafe { _mm256_loadu_ps(right.as_ptr().add(offset)) };
        acc0 = _mm256_fmadd_ps(l, r, acc0);
        offset += 8;
    }

    // 32-lane main loop, 8-lane tail, scalar remainder; tail accumulates into
    // acc0 and is folded back during this reduction.
    let acc01 = _mm256_add_ps(acc0, acc1);
    let acc23 = _mm256_add_ps(acc2, acc3);
    let acc = _mm256_add_ps(acc01, acc23);
    let mut lanes = [0.0_f32; 8];
    unsafe { _mm256_storeu_ps(lanes.as_mut_ptr(), acc) };
    let mut sum = lanes.iter().sum::<f32>();
    for idx in offset..left.len() {
        sum += left[idx] * right[idx];
    }
    sum
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn inner_product_neon(left: &[f32], right: &[f32]) -> f32 {
    let mut acc0: float32x4_t = vdupq_n_f32(0.0);
    let mut acc1: float32x4_t = vdupq_n_f32(0.0);
    let mut acc2: float32x4_t = vdupq_n_f32(0.0);
    let mut acc3: float32x4_t = vdupq_n_f32(0.0);
    let mut offset = 0_usize;

    while offset + 16 <= left.len() {
        let l0 = unsafe { vld1q_f32(left.as_ptr().add(offset)) };
        let r0 = unsafe { vld1q_f32(right.as_ptr().add(offset)) };
        let l1 = unsafe { vld1q_f32(left.as_ptr().add(offset + 4)) };
        let r1 = unsafe { vld1q_f32(right.as_ptr().add(offset + 4)) };
        let l2 = unsafe { vld1q_f32(left.as_ptr().add(offset + 8)) };
        let r2 = unsafe { vld1q_f32(right.as_ptr().add(offset + 8)) };
        let l3 = unsafe { vld1q_f32(left.as_ptr().add(offset + 12)) };
        let r3 = unsafe { vld1q_f32(right.as_ptr().add(offset + 12)) };
        acc0 = vfmaq_f32(acc0, l0, r0);
        acc1 = vfmaq_f32(acc1, l1, r1);
        acc2 = vfmaq_f32(acc2, l2, r2);
        acc3 = vfmaq_f32(acc3, l3, r3);
        offset += 16;
    }

    while offset + 4 <= left.len() {
        let l = unsafe { vld1q_f32(left.as_ptr().add(offset)) };
        let r = unsafe { vld1q_f32(right.as_ptr().add(offset)) };
        acc0 = vfmaq_f32(acc0, l, r);
        offset += 4;
    }

    let acc01 = vaddq_f32(acc0, acc1);
    let acc23 = vaddq_f32(acc2, acc3);
    let acc = vaddq_f32(acc01, acc23);
    let mut lanes = [0.0_f32; 4];
    unsafe { vst1q_f32(lanes.as_mut_ptr(), acc) };
    let mut sum = lanes.iter().sum::<f32>();
    for idx in offset..left.len() {
        sum += left[idx] * right[idx];
    }
    sum
}

pub(crate) unsafe fn resolve_source_attnum(
    heap_relation: pg_sys::Relation,
    source_column: &str,
    source_label: &str,
) -> i32 {
    let source_column = std::ffi::CString::new(source_column)
        .unwrap_or_else(|_| pgrx::error!("ec_hnsw {source_label} contains an invalid NUL byte"));
    let attnum = unsafe { pg_sys::get_attnum((*heap_relation).rd_id, source_column.as_ptr()) };
    let attnum = i32::from(attnum);
    if attnum <= 0 {
        pgrx::error!(
            "ec_hnsw {source_label} \"{}\" does not name a user column on the heap relation",
            source_column.to_string_lossy()
        );
    }
    attnum
}

pub(crate) unsafe fn resolve_source_attribute(
    heap_relation: pg_sys::Relation,
    source_column: &str,
    source_label: &str,
    type_policy: SourceTypePolicy,
) -> SourceAttribute {
    let source_attnum =
        unsafe { resolve_source_attnum(heap_relation, source_column, source_label) };
    unsafe {
        resolve_source_attribute_by_attnum(heap_relation, source_attnum, source_label, type_policy)
    }
}

pub(crate) unsafe fn resolve_source_attribute_by_attnum(
    heap_relation: pg_sys::Relation,
    source_attnum: i32,
    source_label: &str,
    type_policy: SourceTypePolicy,
) -> SourceAttribute {
    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(source_attnum as usize - 1)
        .expect("resolved source attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_hnsw {source_label} references a dropped column");
    }

    let kind = unsafe { resolve_source_datum_kind(att.atttypid) }.unwrap_or_default();
    let valid = match type_policy {
        SourceTypePolicy::BuildSource => {
            matches!(kind, SourceDatumKind::RealArray | SourceDatumKind::Ecvector)
        }
        SourceTypePolicy::RerankSource => matches!(
            kind,
            SourceDatumKind::RealArray | SourceDatumKind::Bytea | SourceDatumKind::Ecvector
        ),
    };
    if !valid {
        let expected = match type_policy {
            SourceTypePolicy::BuildSource => "real[] or ecvector",
            SourceTypePolicy::RerankSource => "real[], bytea, or ecvector",
        };
        pgrx::error!(
            "ec_hnsw {source_label} at heap attnum {} must be {expected}, got type oid {}",
            source_attnum,
            u32::from(att.atttypid),
        );
    }

    SourceAttribute {
        attnum: source_attnum,
        kind,
    }
}

pub(crate) unsafe fn resolve_single_base_heap_index_attnum(
    index_info: *mut pg_sys::IndexInfo,
    label: &str,
) -> i32 {
    if index_info.is_null() {
        pgrx::error!("ec_hnsw {label} received a null IndexInfo");
    }
    let index_info = unsafe { &*index_info };
    if index_info.ii_NumIndexAttrs != 1 || index_info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("ec_hnsw {label} currently supports single-column indexes only");
    }
    if !index_info.ii_Expressions.is_null() {
        pgrx::error!("ec_hnsw {label} does not support expression indexes yet");
    }
    if !index_info.ii_Predicate.is_null() {
        pgrx::error!("ec_hnsw {label} does not support partial indexes yet");
    }

    let attnum = i32::from(index_info.ii_IndexAttrNumbers[0]);
    if attnum <= 0 {
        pgrx::error!("ec_hnsw {label} requires a base heap column index key");
    }
    attnum
}

pub(crate) unsafe fn resolve_indexed_ecvector_attribute_from_index_info(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    label: &str,
) -> SourceAttribute {
    let indexed = unsafe {
        resolve_indexed_vector_attribute_from_index_info(heap_relation, index_info, label)
    };
    if indexed.kind != IndexedVectorKind::Ecvector {
        pgrx::error!("ec_hnsw {label} must be ecvector");
    }
    SourceAttribute {
        attnum: indexed.attnum,
        kind: SourceDatumKind::Ecvector,
    }
}

pub(crate) unsafe fn resolve_indexed_ecvector_attribute(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    label: &str,
) -> SourceAttribute {
    let index_info = unsafe { pg_sys::BuildIndexInfo(index_relation) };
    if index_info.is_null() {
        pgrx::error!("ec_hnsw {label} could not build index metadata");
    }
    let attribute = unsafe {
        resolve_indexed_ecvector_attribute_from_index_info(heap_relation, index_info, label)
    };
    unsafe { pg_sys::pfree(index_info.cast()) };
    attribute
}

pub(crate) unsafe fn resolve_indexed_vector_attribute_from_index_info(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    label: &str,
) -> IndexedVectorAttribute {
    let indexed_attnum = unsafe { resolve_single_base_heap_index_attnum(index_info, label) };
    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(indexed_attnum as usize - 1)
        .expect("resolved indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_hnsw {label} references a dropped column");
    }

    let kind = unsafe { resolve_indexed_vector_kind(att.atttypid) }
        .unwrap_or_else(|| pgrx::error!("ec_hnsw {label} must be ecvector or tqvector"));
    IndexedVectorAttribute {
        attnum: indexed_attnum,
        kind,
    }
}

pub(crate) unsafe fn resolve_indexed_vector_attribute(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    label: &str,
) -> IndexedVectorAttribute {
    let index_info = unsafe { pg_sys::BuildIndexInfo(index_relation) };
    if index_info.is_null() {
        pgrx::error!("ec_hnsw {label} could not build index metadata");
    }
    let attribute = unsafe {
        resolve_indexed_vector_attribute_from_index_info(heap_relation, index_info, label)
    };
    unsafe { pg_sys::pfree(index_info.cast()) };
    attribute
}

unsafe fn resolve_indexed_vector_kind(type_oid: pg_sys::Oid) -> Option<IndexedVectorKind> {
    let base_type_oid = unsafe { pg_sys::getBaseType(type_oid) };
    let formatted = unsafe { pg_sys::format_type_be(base_type_oid) };
    if formatted.is_null() {
        return None;
    }
    let name = unsafe { CStr::from_ptr(formatted) }
        .to_string_lossy()
        .into_owned();
    unsafe { pg_sys::pfree(formatted.cast()) };
    let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
    match type_name {
        "ecvector" => Some(IndexedVectorKind::Ecvector),
        "tqvector" => Some(IndexedVectorKind::Tqvector),
        _ => None,
    }
}

unsafe fn resolve_source_datum_kind(type_oid: pg_sys::Oid) -> Option<SourceDatumKind> {
    match type_oid {
        pg_sys::FLOAT4ARRAYOID => Some(SourceDatumKind::RealArray),
        pg_sys::BYTEAOID => Some(SourceDatumKind::Bytea),
        _ => {
            let base_type_oid = unsafe { pg_sys::getBaseType(type_oid) };
            let formatted = unsafe { pg_sys::format_type_be(base_type_oid) };
            if formatted.is_null() {
                return None;
            }
            let name = unsafe { CStr::from_ptr(formatted) }
                .to_string_lossy()
                .into_owned();
            unsafe { pg_sys::pfree(formatted.cast()) };
            let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
            if type_name == "ecvector" {
                Some(SourceDatumKind::Ecvector)
            } else {
                None
            }
        }
    }
}

pub(crate) unsafe fn allocate_heap_slot(
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

pub(crate) unsafe fn fetch_heap_row_version(
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
            "ec_hnsw {label} could not fetch heap tuple at ({},{})",
            heap_tid.block_number,
            heap_tid.offset_number
        );
    }
}

pub(crate) unsafe fn required_slot_datum(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    label: &str,
) -> pg_sys::Datum {
    if unsafe { (*slot).tts_nvalid } < attnum as i16 {
        unsafe { pg_sys::slot_getsomeattrs_int(slot, attnum) };
    }
    let attr_index = usize::try_from(attnum - 1).expect("attribute number should be positive");
    if unsafe { *(*slot).tts_isnull.add(attr_index) } {
        pgrx::error!("ec_hnsw does not support NULL {label}");
    }
    unsafe { *(*slot).tts_values.add(attr_index) }
}

pub(crate) struct FlatFloat4ArrayRef {
    array_ptr: *mut pg_sys::ArrayType,
    owned: bool,
    data_ptr: *const f32,
    len: usize,
}

impl FlatFloat4ArrayRef {
    pub(crate) unsafe fn from_datum(datum: pg_sys::Datum, label: &str) -> Self {
        if datum.is_null() {
            pgrx::error!("ec_hnsw does not support NULL {label}");
        }

        let original = datum
            .cast_mut_ptr::<std::ffi::c_void>()
            .cast::<pg_sys::varlena>();
        let varlena = unsafe { pg_sys::pg_detoast_datum(original.cast()) };
        if varlena.is_null() {
            pgrx::error!("ec_hnsw could not detoast {label}");
        }
        let array_ptr = varlena.cast::<pg_sys::ArrayType>();
        let owned = !ptr::eq(varlena, original);

        let ndim = match usize::try_from(unsafe { (*array_ptr).ndim }) {
            Ok(value) => value,
            Err(_) => pgrx::error!("ec_hnsw {label} must be a one-dimensional real[]"),
        };
        if ndim != 1 {
            pgrx::error!("ec_hnsw {label} must be a one-dimensional real[]");
        }
        if unsafe { (*array_ptr).elemtype } != pg_sys::FLOAT4OID {
            pgrx::error!("ec_hnsw {label} must be a real[]");
        }
        if unsafe { pg_sys::array_contains_nulls(array_ptr) } {
            pgrx::error!("ec_hnsw {label} arrays must not contain NULL elements");
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
            pgrx::error!("ec_hnsw {label} data pointer is not aligned for float4 access");
        }

        Self {
            array_ptr,
            owned,
            data_ptr,
            len,
        }
    }

    pub(crate) fn as_slice(&self) -> &[f32] {
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

pub(crate) struct FlatFloat4VarlenaRef {
    varlena_ptr: *mut pg_sys::varlena,
    owned: bool,
    data_ptr: *const f32,
    len: usize,
}

impl FlatFloat4VarlenaRef {
    pub(crate) unsafe fn from_datum(datum: pg_sys::Datum, label: &str) -> Self {
        if datum.is_null() {
            pgrx::error!("ec_hnsw does not support NULL {label}");
        }

        let original = datum
            .cast_mut_ptr::<std::ffi::c_void>()
            .cast::<pg_sys::varlena>();
        let varlena = unsafe { pg_sys::pg_detoast_datum(original.cast()) };
        if varlena.is_null() {
            pgrx::error!("ec_hnsw could not detoast {label}");
        }
        let owned = !ptr::eq(varlena, original);
        let bytes = unsafe { pgrx::varlena::varlena_to_byte_slice(varlena) };
        if bytes.len() % std::mem::size_of::<f32>() != 0 {
            pgrx::error!("ec_hnsw {label} bytea payload length must be a multiple of 4 bytes");
        }
        let (prefix, body, suffix) = unsafe { bytes.align_to::<f32>() };
        if !prefix.is_empty() || !suffix.is_empty() {
            pgrx::error!("ec_hnsw {label} bytea payload is not aligned for float4 access");
        }

        Self {
            varlena_ptr: varlena,
            owned,
            data_ptr: body.as_ptr(),
            len: body.len(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.len) }
    }
}

impl Drop for FlatFloat4VarlenaRef {
    fn drop(&mut self) {
        if self.owned {
            unsafe { pg_sys::pfree(self.varlena_ptr.cast()) };
        }
    }
}

pub(crate) enum FlatFloat4SourceRef {
    Array(FlatFloat4ArrayRef),
    Varlena(FlatFloat4VarlenaRef),
}

impl FlatFloat4SourceRef {
    pub(crate) unsafe fn from_datum(
        datum: pg_sys::Datum,
        kind: SourceDatumKind,
        label: &str,
    ) -> Self {
        match kind {
            SourceDatumKind::RealArray => {
                Self::Array(unsafe { FlatFloat4ArrayRef::from_datum(datum, label) })
            }
            SourceDatumKind::Bytea | SourceDatumKind::Ecvector => {
                Self::Varlena(unsafe { FlatFloat4VarlenaRef::from_datum(datum, label) })
            }
            _ => pgrx::error!("ec_hnsw {label} must be real[], bytea, or ecvector"),
        }
    }

    pub(crate) fn as_slice(&self) -> &[f32] {
        match self {
            Self::Array(array) => array.as_slice(),
            Self::Varlena(varlena) => varlena.as_slice(),
        }
    }
}

pub(crate) unsafe fn load_source_from_heap_row(
    heap_relation: pg_sys::Relation,
    heap_tid: page::ItemPointer,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attribute: SourceAttribute,
    label: &str,
) -> FlatFloat4SourceRef {
    unsafe { fetch_heap_row_version(heap_relation, heap_tid, snapshot, slot, label) };
    let source_datum = unsafe { required_slot_datum(slot, source_attribute.attnum, label) };
    unsafe { FlatFloat4SourceRef::from_datum(source_datum, source_attribute.kind, label) }
}

pub(crate) fn negative_inner_product(query: &[f32], source: &[f32]) -> f32 {
    if query.len() != source.len() {
        pgrx::error!(
            "ec_hnsw source vector dimension mismatch: left dim {}, right dim {}",
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

pub(crate) fn negative_inner_product_index_internal(query: &[f32], source: &[f32]) -> f32 {
    if query.len() != source.len() {
        pgrx::error!(
            "ec_hnsw source vector dimension mismatch: left dim {}, right dim {}",
            query.len(),
            source.len()
        );
    }
    -inner_product(query, source)
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

    #[test]
    fn negative_inner_product_index_internal_matches_scalar_reference() {
        let left = (0..1536)
            .map(|idx| (idx as f32 * 0.017).sin())
            .collect::<Vec<_>>();
        let right = (0..1536)
            .map(|idx| (idx as f32 * 0.031).cos())
            .collect::<Vec<_>>();
        let expected = -inner_product_scalar(&left, &right);
        let actual = negative_inner_product_index_internal(&left, &right);

        assert!(
            (actual - expected).abs() <= 0.0005,
            "actual={actual} expected={expected}"
        );
    }

    #[test]
    fn inner_product_matches_scalar_reference_for_tail_lengths() {
        for len in (0..19).chain([41]) {
            let left = (0..len)
                .map(|idx| idx as f32 * 0.25 - 1.5)
                .collect::<Vec<_>>();
            let right = (0..len)
                .map(|idx| (idx as f32 * 0.125).sin())
                .collect::<Vec<_>>();
            let expected = inner_product_scalar(&left, &right);
            let actual = inner_product(&left, &right);

            assert!(
                (actual - expected).abs() <= 0.00001,
                "len={len} actual={actual} expected={expected}"
            );
        }
    }

    #[test]
    fn inner_product_matches_scalar_reference_for_real_dimension() {
        let left = (0..1536)
            .map(|idx| (idx as f32 * 0.017).sin())
            .collect::<Vec<_>>();
        let right = (0..1536)
            .map(|idx| (idx as f32 * 0.031).cos())
            .collect::<Vec<_>>();
        let expected = inner_product_scalar(&left, &right);
        let actual = inner_product(&left, &right);

        assert!(
            (actual - expected).abs() <= 0.0005,
            "actual={actual} expected={expected}"
        );
    }
}
