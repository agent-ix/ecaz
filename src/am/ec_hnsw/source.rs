use pgrx::pg_sys;

use crate::am::common::{
    datum::{AttnumLookup, FlatFloat4Kind, FlatFloat4Source},
    heap_slot,
};

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

pub(crate) fn resolve_source_attnum(
    heap_relation: pg_sys::Relation,
    source_column: &str,
    source_label: &str,
) -> i32 {
    let attnum = AttnumLookup::lookup(heap_relation, source_column).unwrap_or_else(|| {
        pgrx::error!(
            "ec_hnsw {source_label} \"{source_column}\" does not name a user column on the heap relation"
        )
    });
    i32::from(attnum)
}

pub(crate) fn resolve_source_attribute(
    heap_relation: pg_sys::Relation,
    source_column: &str,
    source_label: &str,
    type_policy: SourceTypePolicy,
) -> SourceAttribute {
    let source_attnum = resolve_source_attnum(heap_relation, source_column, source_label);
    resolve_source_attribute_by_attnum(heap_relation, source_attnum, source_label, type_policy)
}

pub(crate) fn resolve_source_attribute_by_attnum(
    heap_relation: pg_sys::Relation,
    source_attnum: i32,
    source_label: &str,
    type_policy: SourceTypePolicy,
) -> SourceAttribute {
    let heap_relation = std::ptr::NonNull::new(heap_relation)
        .unwrap_or_else(|| pgrx::error!("ec_hnsw source resolution needs a valid heap relation"));
    let tuple_desc = crate::storage::relation::relation_tuple_desc_copy_handle(heap_relation);
    let att = tuple_desc
        .get(source_attnum as usize - 1)
        .expect("resolved source attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_hnsw {source_label} references a dropped column");
    }

    // SAFETY: `att.atttypid` comes from the copied tuple descriptor metadata.
    let kind = resolve_source_datum_kind(att.atttypid).unwrap_or_default();
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

pub(crate) fn resolve_single_base_heap_index_attnum(
    index_info: *mut pg_sys::IndexInfo,
    label: &str,
) -> i32 {
    if index_info.is_null() {
        pgrx::error!("ec_hnsw {label} received a null IndexInfo");
    }
    let index_info = crate::am::common::pg_ptr::index_info(
        std::ptr::NonNull::new(index_info).expect("ec_hnsw IndexInfo should be non-null"),
    );
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

pub(crate) fn resolve_indexed_ecvector_attribute_from_index_info(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    label: &str,
) -> SourceAttribute {
    // SAFETY: The heap relation is live and `index_info` is callback-duration
    // metadata owned by PostgreSQL.
    let indexed =
        resolve_indexed_vector_attribute_from_index_info(heap_relation, index_info, label);
    if indexed.kind != IndexedVectorKind::Ecvector {
        pgrx::error!("ec_hnsw {label} must be ecvector");
    }
    SourceAttribute {
        attnum: indexed.attnum,
        kind: SourceDatumKind::Ecvector,
    }
}

pub(crate) fn resolve_indexed_ecvector_attribute(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    label: &str,
) -> SourceAttribute {
    let index_info = super::index_info::IndexInfoGuard::build(index_relation, label);
    // SAFETY: `index_info` was checked non-null and belongs to this index.
    let attribute = resolve_indexed_ecvector_attribute_from_index_info(
        heap_relation,
        index_info.as_ptr(),
        label,
    );
    attribute
}

pub(crate) fn resolve_indexed_vector_attribute_from_index_info(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    label: &str,
) -> IndexedVectorAttribute {
    // SAFETY: `index_info` is callback-duration PostgreSQL metadata and the
    // helper validates single-key base-column shape.
    let indexed_attnum = resolve_single_base_heap_index_attnum(index_info, label);
    let heap_relation = std::ptr::NonNull::new(heap_relation)
        .unwrap_or_else(|| pgrx::error!("ec_hnsw source resolution needs a valid heap relation"));
    let tuple_desc = crate::storage::relation::relation_tuple_desc_copy_handle(heap_relation);
    let att = tuple_desc
        .get(indexed_attnum as usize - 1)
        .expect("resolved indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_hnsw {label} references a dropped column");
    }

    // SAFETY: `att.atttypid` comes from the copied tuple descriptor metadata.
    let kind = resolve_indexed_vector_kind(att.atttypid)
        .unwrap_or_else(|| pgrx::error!("ec_hnsw {label} must be ecvector or tqvector"));
    IndexedVectorAttribute {
        attnum: indexed_attnum,
        kind,
    }
}

pub(crate) fn resolve_indexed_vector_attribute(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    label: &str,
) -> IndexedVectorAttribute {
    let index_info = super::index_info::IndexInfoGuard::build(index_relation, label);
    // SAFETY: `index_info` was checked non-null and belongs to this index.
    let attribute =
        resolve_indexed_vector_attribute_from_index_info(heap_relation, index_info.as_ptr(), label);
    attribute
}

fn resolve_indexed_vector_kind(type_oid: pg_sys::Oid) -> Option<IndexedVectorKind> {
    let name = crate::storage::type_info::formatted_base_type_name(type_oid)?;
    let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
    match type_name {
        "ecvector" => Some(IndexedVectorKind::Ecvector),
        "tqvector" => Some(IndexedVectorKind::Tqvector),
        _ => None,
    }
}

fn resolve_source_datum_kind(type_oid: pg_sys::Oid) -> Option<SourceDatumKind> {
    match type_oid {
        pg_sys::FLOAT4ARRAYOID => Some(SourceDatumKind::RealArray),
        pg_sys::BYTEAOID => Some(SourceDatumKind::Bytea),
        _ => {
            let name = crate::storage::type_info::formatted_base_type_name(type_oid)?;
            let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
            if type_name == "ecvector" {
                Some(SourceDatumKind::Ecvector)
            } else {
                None
            }
        }
    }
}

pub(crate) fn fetch_heap_row_version_with_reader(
    reader: &mut heap_slot::HeapSlotReader<'_>,
    heap_tid: page::ItemPointer,
    label: &str,
) {
    let fetched = reader
        .fetch_row_version(heap_tid)
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    if !fetched {
        pgrx::error!(
            "ec_hnsw {label} could not fetch heap tuple at ({},{})",
            heap_tid.block_number,
            heap_tid.offset_number
        );
    }
}

pub(crate) fn required_slot_datum_with_reader(
    reader: &mut heap_slot::HeapSlotReader<'_>,
    attnum: i32,
    label: &str,
) -> pg_sys::Datum {
    reader
        .required_datum(attnum, label)
        .unwrap_or_else(|error| pgrx::error!("{error}"))
}

/// Map an HNSW [`SourceDatumKind`] to the common-layer [`FlatFloat4Kind`].
///
/// Errors via `pgrx::error!` on `Unknown` since unresolved source datum kinds
/// indicate a build-time metadata bug, not a recoverable input shape.
fn flat_float4_kind(kind: SourceDatumKind, label: &str) -> FlatFloat4Kind {
    match kind {
        SourceDatumKind::RealArray => FlatFloat4Kind::RealArray,
        SourceDatumKind::Bytea | SourceDatumKind::Ecvector => FlatFloat4Kind::Varlena,
        SourceDatumKind::Unknown => {
            pgrx::error!("ec_hnsw {label} must be real[], bytea, or ecvector")
        }
    }
}

/// Closure-CPS entry that materialises a [`FlatFloat4Source`] over `datum` and
/// passes the resulting borrowed view to `f`. The higher-ranked closure keeps
/// Datum-backed slices local to this call: callers may copy or score from them
/// but cannot return the borrowed view.
///
/// # Safety
/// `datum` must be a live PostgreSQL Datum whose static type matches `kind`
/// (resolved upstream via [`resolve_source_attribute`] /
/// [`resolve_indexed_vector_attribute`]). The backing varlena must outlive the
/// closure invocation.
pub(crate) unsafe fn with_flat_float4_source_from_datum<R>(
    datum: pg_sys::Datum,
    kind: SourceDatumKind,
    label: &str,
    f: impl for<'datum> FnOnce(FlatFloat4Source<'datum>) -> R,
) -> R {
    let flat_kind = flat_float4_kind(kind, label);
    // SAFETY: `flat_kind` is derived from `kind`, which the caller has
    // type-checked against the live PostgreSQL relation metadata; the closure
    // lifetime prevents the borrowed datum view from escaping this call.
    let source = unsafe { FlatFloat4Source::from_datum(datum, flat_kind, label) }
        .unwrap_or_else(|| pgrx::error!("ec_hnsw does not support NULL {label}"));
    f(source)
}

pub(crate) fn with_source_from_heap_row_reader<R>(
    reader: &mut heap_slot::HeapSlotReader<'_>,
    heap_tid: page::ItemPointer,
    source_attribute: SourceAttribute,
    label: &str,
    f: impl for<'datum> FnOnce(FlatFloat4Source<'datum>) -> R,
) -> R {
    fetch_heap_row_version_with_reader(reader, heap_tid, label);
    let source_datum = required_slot_datum_with_reader(reader, source_attribute.attnum, label);
    // SAFETY: The source kind was resolved from heap metadata and the closure
    // keeps the datum-backed source view scoped to this call.
    unsafe { with_flat_float4_source_from_datum(source_datum, source_attribute.kind, label, f) }
}

pub(crate) fn with_indexed_ecvector_from_slot_reader<R>(
    reader: &mut heap_slot::HeapSlotReader<'_>,
    attnum: i32,
    label: &str,
    f: impl for<'datum> FnOnce(FlatFloat4Source<'datum>) -> R,
) -> R {
    let source_datum = required_slot_datum_with_reader(reader, attnum, label);
    // SAFETY: The indexed attribute is required to be ecvector, which is stored
    // as a byte-backed varlena float payload; the closure scopes the borrow.
    let source =
        unsafe { FlatFloat4Source::from_datum(source_datum, FlatFloat4Kind::Varlena, label) }
            .unwrap_or_else(|| pgrx::error!("ec_hnsw does not support NULL {label}"));
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
