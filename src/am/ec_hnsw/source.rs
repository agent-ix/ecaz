use std::{
    ffi::{c_int, CStr},
    marker::PhantomData,
};

use pgrx::{pg_sys, PgTupleDesc};

use crate::am::common::{detoast::DetoastedVarlena, heap_slot};

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
            // SAFETY: Runtime feature detection guarantees AVX2/FMA support and
            // the function slices both operands to the same minimum length.
            return unsafe { inner_product_avx2_fma(left, right) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: Runtime feature detection guarantees NEON support and the
            // function slices both operands to the same minimum length.
            return unsafe { inner_product_neon(left, right) };
        }
    }

    inner_product_scalar(left, right)
}

#[cfg(any(test, feature = "bench"))]
pub(crate) fn inner_product_scalar_reference(left: &[f32], right: &[f32]) -> f32 {
    inner_product_scalar(left, right)
}

#[cfg(all(
    any(test, feature = "bench"),
    any(target_arch = "x86", target_arch = "x86_64")
))]
pub(crate) fn inner_product_avx2_fma_for_test(left: &[f32], right: &[f32]) -> Option<f32> {
    if !std::arch::is_x86_feature_detected!("avx2") || !std::arch::is_x86_feature_detected!("fma") {
        return None;
    }
    // SAFETY: The test helper returns `None` unless AVX2/FMA are available and
    // forwards caller-owned same-length test slices.
    Some(unsafe { inner_product_avx2_fma(left, right) })
}

#[cfg(all(any(test, feature = "bench"), target_arch = "aarch64"))]
pub(crate) fn inner_product_neon_for_test(left: &[f32], right: &[f32]) -> Option<f32> {
    if !std::arch::is_aarch64_feature_detected!("neon") {
        return None;
    }
    // SAFETY: The test helper returns `None` unless NEON is available and
    // forwards caller-owned same-length test slices.
    Some(unsafe { inner_product_neon(left, right) })
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
        // SAFETY: The loop guard leaves at least 32 f32 lanes available from
        // `offset`; unaligned AVX loads accept any valid f32 address.
        let (l0, r0, l1, r1, l2, r2, l3, r3) = unsafe {
            (
                _mm256_loadu_ps(left.as_ptr().add(offset)),
                _mm256_loadu_ps(right.as_ptr().add(offset)),
                _mm256_loadu_ps(left.as_ptr().add(offset + 8)),
                _mm256_loadu_ps(right.as_ptr().add(offset + 8)),
                _mm256_loadu_ps(left.as_ptr().add(offset + 16)),
                _mm256_loadu_ps(right.as_ptr().add(offset + 16)),
                _mm256_loadu_ps(left.as_ptr().add(offset + 24)),
                _mm256_loadu_ps(right.as_ptr().add(offset + 24)),
            )
        };
        acc0 = _mm256_fmadd_ps(l0, r0, acc0);
        acc1 = _mm256_fmadd_ps(l1, r1, acc1);
        acc2 = _mm256_fmadd_ps(l2, r2, acc2);
        acc3 = _mm256_fmadd_ps(l3, r3, acc3);
        offset += 32;
    }
    while offset + 8 <= left.len() {
        // SAFETY: The tail loop guard leaves at least 8 f32 lanes available
        // from `offset`; unaligned AVX loads accept any valid f32 address.
        let (l, r) = unsafe {
            (
                _mm256_loadu_ps(left.as_ptr().add(offset)),
                _mm256_loadu_ps(right.as_ptr().add(offset)),
            )
        };
        acc0 = _mm256_fmadd_ps(l, r, acc0);
        offset += 8;
    }

    // 32-lane main loop, 8-lane tail, scalar remainder; tail accumulates into
    // acc0 and is folded back during this reduction.
    let acc01 = _mm256_add_ps(acc0, acc1);
    let acc23 = _mm256_add_ps(acc2, acc3);
    let acc = _mm256_add_ps(acc01, acc23);
    let mut lanes = [0.0_f32; 8];
    // SAFETY: `lanes` has exactly eight f32 slots, matching one AVX register.
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
        // SAFETY: The loop guard leaves at least 16 f32 lanes available from
        // `offset`, and NEON support is guaranteed by the caller.
        let (l0, r0, l1, r1, l2, r2, l3, r3) = unsafe {
            (
                vld1q_f32(left.as_ptr().add(offset)),
                vld1q_f32(right.as_ptr().add(offset)),
                vld1q_f32(left.as_ptr().add(offset + 4)),
                vld1q_f32(right.as_ptr().add(offset + 4)),
                vld1q_f32(left.as_ptr().add(offset + 8)),
                vld1q_f32(right.as_ptr().add(offset + 8)),
                vld1q_f32(left.as_ptr().add(offset + 12)),
                vld1q_f32(right.as_ptr().add(offset + 12)),
            )
        };
        acc0 = vfmaq_f32(acc0, l0, r0);
        acc1 = vfmaq_f32(acc1, l1, r1);
        acc2 = vfmaq_f32(acc2, l2, r2);
        acc3 = vfmaq_f32(acc3, l3, r3);
        offset += 16;
    }

    while offset + 4 <= left.len() {
        // SAFETY: The tail loop guard leaves at least 4 f32 lanes available
        // from `offset`, and NEON support is guaranteed by the caller.
        let (l, r) = unsafe {
            (
                vld1q_f32(left.as_ptr().add(offset)),
                vld1q_f32(right.as_ptr().add(offset)),
            )
        };
        acc0 = vfmaq_f32(acc0, l, r);
        offset += 4;
    }

    let acc01 = vaddq_f32(acc0, acc1);
    let acc23 = vaddq_f32(acc2, acc3);
    let acc = vaddq_f32(acc01, acc23);
    let mut lanes = [0.0_f32; 4];
    // SAFETY: `lanes` has exactly four f32 slots, matching one NEON register.
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
    // SAFETY: The heap relation is live for the caller's PostgreSQL callback,
    // and `source_column` is a NUL-terminated CString for `get_attnum`.
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
    // SAFETY: The caller supplies a live heap relation and column label; this
    // helper validates the resolved attnum before it is reused below.
    let source_attnum =
        unsafe { resolve_source_attnum(heap_relation, source_column, source_label) };
    // SAFETY: `source_attnum` was resolved from this heap relation and type
    // policy validation happens inside the delegated helper.
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
    // SAFETY: The heap relation is live for the caller's PostgreSQL callback;
    // `from_pg_copy` copies the tuple descriptor metadata before inspection.
    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(source_attnum as usize - 1)
        .expect("resolved source attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_hnsw {source_label} references a dropped column");
    }

    // SAFETY: `att.atttypid` comes from the copied tuple descriptor metadata.
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
    // SAFETY: Null was checked above and PostgreSQL owns `IndexInfo` for the
    // duration of the calling AM callback.
    let index_info = unsafe { &*index_info };
    if index_info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("ec_hnsw {label} currently supports single-key indexes only");
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
    // SAFETY: The heap relation is live and `index_info` is callback-duration
    // metadata owned by PostgreSQL.
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
    // SAFETY: The index relation is live; BuildIndexInfo returns palloc'd
    // metadata for this relation.
    let index_info = unsafe { pg_sys::BuildIndexInfo(index_relation) };
    if index_info.is_null() {
        pgrx::error!("ec_hnsw {label} could not build index metadata");
    }
    // SAFETY: `index_info` was checked non-null and belongs to this index.
    let attribute = unsafe {
        resolve_indexed_ecvector_attribute_from_index_info(heap_relation, index_info, label)
    };
    // SAFETY: `index_info` was allocated by PostgreSQL BuildIndexInfo above.
    unsafe { pg_sys::pfree(index_info.cast()) };
    attribute
}

pub(crate) unsafe fn resolve_indexed_vector_attribute_from_index_info(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    label: &str,
) -> IndexedVectorAttribute {
    // SAFETY: `index_info` is callback-duration PostgreSQL metadata and the
    // helper validates single-key base-column shape.
    let indexed_attnum = unsafe { resolve_single_base_heap_index_attnum(index_info, label) };
    // SAFETY: The heap relation is live; `from_pg_copy` copies tuple descriptor
    // metadata before the indexed attribute is inspected.
    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(indexed_attnum as usize - 1)
        .expect("resolved indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_hnsw {label} references a dropped column");
    }

    // SAFETY: `att.atttypid` comes from the copied tuple descriptor metadata.
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
    // SAFETY: The index relation is live; BuildIndexInfo returns palloc'd
    // metadata for this relation.
    let index_info = unsafe { pg_sys::BuildIndexInfo(index_relation) };
    if index_info.is_null() {
        pgrx::error!("ec_hnsw {label} could not build index metadata");
    }
    // SAFETY: `index_info` was checked non-null and belongs to this index.
    let attribute = unsafe {
        resolve_indexed_vector_attribute_from_index_info(heap_relation, index_info, label)
    };
    // SAFETY: `index_info` was allocated by PostgreSQL BuildIndexInfo above.
    unsafe { pg_sys::pfree(index_info.cast()) };
    attribute
}

unsafe fn resolve_indexed_vector_kind(type_oid: pg_sys::Oid) -> Option<IndexedVectorKind> {
    let name = formatted_base_type_name(type_oid)?;
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
            let name = formatted_base_type_name(type_oid)?;
            let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
            if type_name == "ecvector" {
                Some(SourceDatumKind::Ecvector)
            } else {
                None
            }
        }
    }
}

fn formatted_base_type_name(type_oid: pg_sys::Oid) -> Option<String> {
    // SAFETY: PostgreSQL accepts any type OID here, `format_type_be` returns a
    // palloc'd NUL-terminated string for known type OIDs, and that allocation
    // is released before the copied Rust string is returned.
    unsafe {
        let base_type_oid = pg_sys::getBaseType(type_oid);
        let formatted = pg_sys::format_type_be(base_type_oid);
        if formatted.is_null() {
            return None;
        }
        let name = CStr::from_ptr(formatted).to_string_lossy().into_owned();
        pg_sys::pfree(formatted.cast());
        Some(name)
    }
}

pub(crate) unsafe fn fetch_heap_row_version(
    heap_relation: pg_sys::Relation,
    heap_tid: page::ItemPointer,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    label: &str,
) {
    // SAFETY: caller owns the heap relation, snapshot, and tuple slot for the
    // current scan/build/vacuum callback.
    let fetched = unsafe {
        heap_slot::fetch_heap_row_version(heap_relation, heap_tid, snapshot, slot, "ec_hnsw")
    }
    .unwrap_or_else(|error| pgrx::error!("{error}"));
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
    // SAFETY: caller owns a live TupleTableSlot and attnum was resolved from
    // relation metadata for the source column.
    unsafe { heap_slot::required_slot_datum(slot, attnum, "ec_hnsw", label) }
        .unwrap_or_else(|error| pgrx::error!("{error}"))
}

struct DetoastedFloat4Datum {
    varlena: DetoastedVarlena,
}

impl DetoastedFloat4Datum {
    unsafe fn from_datum(datum: pg_sys::Datum, label: &str) -> Self {
        if datum.is_null() {
            pgrx::error!("ec_hnsw does not support NULL {label}");
        }

        // SAFETY: The datum is non-null and expected to be a varlena float
        // payload selected by earlier type validation.
        let varlena = unsafe { DetoastedVarlena::plain_from_datum(datum) }
            .unwrap_or_else(|| pgrx::error!("ec_hnsw could not detoast {label}"));

        Self { varlena }
    }

    fn as_array_ptr(&self) -> *mut pg_sys::ArrayType {
        self.varlena.as_ptr().cast::<pg_sys::ArrayType>()
    }

    fn as_bytes(&self) -> &[u8] {
        self.varlena.as_bytes()
    }
}

pub(crate) struct FlatFloat4ArrayRef<'datum> {
    _detoasted: DetoastedFloat4Datum,
    data_ptr: *const f32,
    len: usize,
    _datum: PhantomData<&'datum [f32]>,
}

impl<'datum> FlatFloat4ArrayRef<'datum> {
    unsafe fn from_datum(datum: pg_sys::Datum, label: &str) -> Self {
        if datum.is_null() {
            pgrx::error!("ec_hnsw does not support NULL {label}");
        }

        // SAFETY: The caller has already type-checked this datum as a supported
        // source value and this helper owns the detoasted backing storage.
        let detoasted = unsafe { DetoastedFloat4Datum::from_datum(datum, label) };
        let array_ptr = detoasted.as_array_ptr();

        // SAFETY: `array_ptr` points at the detoasted ArrayType backing storage.
        let ndim = match usize::try_from(unsafe { (*array_ptr).ndim }) {
            Ok(value) => value,
            Err(_) => pgrx::error!("ec_hnsw {label} must be a one-dimensional real[]"),
        };
        if ndim != 1 {
            pgrx::error!("ec_hnsw {label} must be a one-dimensional real[]");
        }
        // SAFETY: `array_ptr` is the detoasted ArrayType and `elemtype` is part
        // of the fixed array header.
        if unsafe { (*array_ptr).elemtype } != pg_sys::FLOAT4OID {
            pgrx::error!("ec_hnsw {label} must be a real[]");
        }
        // SAFETY: `array_ptr` is a valid detoasted ArrayType.
        if unsafe { pg_sys::array_contains_nulls(array_ptr) } {
            pgrx::error!("ec_hnsw {label} arrays must not contain NULL elements");
        }

        // SAFETY: The array is a detoasted one-dimensional flat ArrayType.
        let dims_ptr = unsafe { flat_array_dims_ptr(array_ptr) };
        // SAFETY: `ndim` and `dims_ptr` come from the same ArrayType header.
        let len = usize::try_from(unsafe { pg_sys::ArrayGetNItems((*array_ptr).ndim, dims_ptr) })
            .expect("flat float4 array length should fit in usize");
        // SAFETY: Data offset is computed from the same flat ArrayType header;
        // alignment is checked before exposing the f32 slice.
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
            _detoasted: detoasted,
            data_ptr,
            len,
            _datum: PhantomData,
        }
    }

    pub(crate) fn as_slice(&self) -> &[f32] {
        // SAFETY: `data_ptr` and `len` were validated during construction and
        // the detoasted backing storage is owned by `self`.
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.len) }
    }
}

pub(crate) struct FlatFloat4VarlenaRef<'datum> {
    _detoasted: DetoastedFloat4Datum,
    data_ptr: *const f32,
    len: usize,
    _datum: PhantomData<&'datum [f32]>,
}

impl<'datum> FlatFloat4VarlenaRef<'datum> {
    unsafe fn from_datum(datum: pg_sys::Datum, label: &str) -> Self {
        if datum.is_null() {
            pgrx::error!("ec_hnsw does not support NULL {label}");
        }

        // SAFETY: The caller has already type-checked this datum as a supported
        // byte-backed source value and this helper owns the detoasted backing.
        let detoasted = unsafe { DetoastedFloat4Datum::from_datum(datum, label) };
        let (data_ptr, len) = {
            let bytes = detoasted.as_bytes();
            if bytes.len() % std::mem::size_of::<f32>() != 0 {
                pgrx::error!("ec_hnsw {label} bytea payload length must be a multiple of 4 bytes");
            }
            // SAFETY: `align_to` is used only to validate exact f32 alignment;
            // any non-empty prefix/suffix is rejected before the body is stored.
            let (prefix, body, suffix) = unsafe { bytes.align_to::<f32>() };
            if !prefix.is_empty() || !suffix.is_empty() {
                pgrx::error!("ec_hnsw {label} bytea payload is not aligned for float4 access");
            }
            (body.as_ptr(), body.len())
        };

        Self {
            _detoasted: detoasted,
            data_ptr,
            len,
            _datum: PhantomData,
        }
    }

    pub(crate) fn as_slice(&self) -> &[f32] {
        // SAFETY: `data_ptr` and `len` were validated during construction and
        // the detoasted backing storage is owned by `self`.
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.len) }
    }
}

pub(crate) enum FlatFloat4SourceRef<'datum> {
    Array(FlatFloat4ArrayRef<'datum>),
    Varlena(FlatFloat4VarlenaRef<'datum>),
}

impl<'datum> FlatFloat4SourceRef<'datum> {
    unsafe fn from_datum(datum: pg_sys::Datum, kind: SourceDatumKind, label: &str) -> Self {
        match kind {
            SourceDatumKind::RealArray => {
                // SAFETY: `kind` records that the datum was type-checked as a
                // supported real[] source before dispatch.
                Self::Array(unsafe { FlatFloat4ArrayRef::from_datum(datum, label) })
            }
            SourceDatumKind::Bytea | SourceDatumKind::Ecvector => {
                // SAFETY: `kind` records that the datum was type-checked as a
                // supported byte-backed source before dispatch.
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

pub(crate) unsafe fn with_flat_float4_source_from_datum<R>(
    datum: pg_sys::Datum,
    kind: SourceDatumKind,
    label: &str,
    f: impl for<'datum> FnOnce(FlatFloat4SourceRef<'datum>) -> R,
) -> R {
    // The higher-ranked closure keeps Datum-backed slices local to this call:
    // callers may copy or score from them, but cannot return the borrowed view.
    // SAFETY: `kind` is resolved from PostgreSQL type metadata before this call
    // and the closure lifetime prevents the borrowed datum view escaping.
    let source = unsafe { FlatFloat4SourceRef::from_datum(datum, kind, label) };
    f(source)
}

pub(crate) unsafe fn with_source_from_heap_row<R>(
    heap_relation: pg_sys::Relation,
    heap_tid: page::ItemPointer,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attribute: SourceAttribute,
    label: &str,
    f: impl for<'datum> FnOnce(FlatFloat4SourceRef<'datum>) -> R,
) -> R {
    // SAFETY: The heap relation/snapshot/slot are caller-owned for this
    // callback and `heap_tid` came from the index tuple being examined.
    unsafe { fetch_heap_row_version(heap_relation, heap_tid, snapshot, slot, label) };
    // SAFETY: The slot now holds the requested row version and the source
    // attnum was resolved from heap metadata.
    let source_datum = unsafe { required_slot_datum(slot, source_attribute.attnum, label) };
    // SAFETY: The source kind was resolved from heap metadata and the closure
    // keeps the datum-backed source view scoped to this call.
    unsafe { with_flat_float4_source_from_datum(source_datum, source_attribute.kind, label, f) }
}

pub(crate) unsafe fn with_indexed_ecvector_from_slot<R>(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    label: &str,
    f: impl for<'datum> FnOnce(FlatFloat4VarlenaRef<'datum>) -> R,
) -> R {
    // SAFETY: The slot contains a row with the indexed ecvector attribute and
    // `attnum` was resolved from index/heap metadata.
    let source_datum = unsafe { required_slot_datum(slot, attnum, label) };
    // SAFETY: The indexed attribute is required to be ecvector, which is stored
    // as a byte-backed varlena float payload.
    let source = unsafe { FlatFloat4VarlenaRef::from_datum(source_datum, label) };
    f(source)
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
    // SAFETY: The caller supplies a detoasted flat ArrayType pointer; array dims
    // immediately follow the fixed ArrayType header in PostgreSQL layout.
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
    // SAFETY: The caller supplies a detoasted ArrayType pointer and `dataoffset`
    // is a fixed header field.
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
