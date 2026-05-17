//! `ec_diskann` is the Vamana-based secondary access method.
//!
//! The module owns the AM callback surface plus the persisted Vamana
//! helpers that support build, ordered scan, live insert, vacuum
//! repair, and planner costing for grouped-PQ-backed `ecvector`
//! indexes.

mod ambuild;
pub mod build;
mod cost;
pub mod diagnostics;
mod insert;
mod options;
pub mod page;
pub mod persist;
pub mod reader;
mod routine;
pub mod scan;
pub mod scan_query;
mod scan_state;
pub mod tuple;
pub mod vacuum;
pub mod vamana;

pub(crate) fn register_gucs() {
    options::register_gucs();
}

pub(super) const ECDISKANN_DEFAULT_GRAPH_DEGREE: i32 = 32;
pub(super) const ECDISKANN_MIN_GRAPH_DEGREE: i32 = 4;
pub(super) const ECDISKANN_MAX_GRAPH_DEGREE: i32 = 256;

pub(super) const ECDISKANN_DEFAULT_BUILD_LIST_SIZE: i32 = 100;
pub(super) const ECDISKANN_MIN_BUILD_LIST_SIZE: i32 = 10;
pub(super) const ECDISKANN_MAX_BUILD_LIST_SIZE: i32 = 1000;

pub(super) const ECDISKANN_DEFAULT_SCAN_LIST_SIZE: i32 = 100;
pub(super) const ECDISKANN_MIN_SCAN_LIST_SIZE: i32 = 1;
pub(super) const ECDISKANN_MAX_SCAN_LIST_SIZE: i32 = 10_000;

pub(super) const ECDISKANN_DEFAULT_RERANK_BUDGET: i32 = 64;
pub(super) const ECDISKANN_MIN_RERANK_BUDGET: i32 = 1;
pub(super) const ECDISKANN_MAX_RERANK_BUDGET: i32 = 10_000;

pub(super) const ECDISKANN_DEFAULT_TOP_K: i32 = 10;
pub(super) const ECDISKANN_MIN_TOP_K: i32 = 1;
pub(super) const ECDISKANN_MAX_TOP_K: i32 = 10_000;

pub(super) const ECDISKANN_DEFAULT_ALPHA: f32 = 1.2;
pub(super) const ECDISKANN_MIN_ALPHA: f32 = 1.0;
pub(super) const ECDISKANN_MAX_ALPHA: f32 = 2.0;

pub(super) const ECDISKANN_PLANNER_SCAN_ENABLED: bool = true;
// V0 exact graph distances use `1 - ip`, which only preserves the `<#>`
// ordering when the source vectors are unit-normalized.
pub(super) const ECDISKANN_UNIT_NORM_DISTANCE_BIAS: f32 = 1.0;
pub(super) const ECDISKANN_UNIT_NORM_EPSILON: f32 = 0.01;
pub(super) const ECDISKANN_UNIT_NORM_BUILD_SAMPLE_CAP: usize = 1024;

pub(super) fn validate_source_vector_unit_norm(
    source_vector: &[f32],
    context: &str,
) -> Result<(), String> {
    let norm = source_vector_l2_norm(source_vector);
    if !norm.is_finite() {
        return Err(format!(
            "ec_diskann {context} requires finite unit-normalized source vectors; got ||v|| = {norm}"
        ));
    }
    let low = ECDISKANN_UNIT_NORM_DISTANCE_BIAS - ECDISKANN_UNIT_NORM_EPSILON;
    let high = ECDISKANN_UNIT_NORM_DISTANCE_BIAS + ECDISKANN_UNIT_NORM_EPSILON;
    if !(low..=high).contains(&norm) {
        return Err(format!(
            "ec_diskann {context} requires unit-normalized source vectors for the v0 distance wrapper; got ||v|| = {norm:.4}, expected within [{low:.4}, {high:.4}]"
        ));
    }
    Ok(())
}

pub(super) fn validate_source_vector_unit_norm_sample(
    source_vectors: &[&[f32]],
    sample_cap: usize,
    context: &str,
) -> Result<(), String> {
    let sample_len = source_vectors.len().min(sample_cap);
    if sample_len == 0 {
        return Ok(());
    }

    let mut norm_sum = 0.0_f32;
    let mut min_norm = f32::INFINITY;
    let mut max_norm = f32::NEG_INFINITY;
    let mut first_outlier = None;
    let low = ECDISKANN_UNIT_NORM_DISTANCE_BIAS - ECDISKANN_UNIT_NORM_EPSILON;
    let high = ECDISKANN_UNIT_NORM_DISTANCE_BIAS + ECDISKANN_UNIT_NORM_EPSILON;

    for (idx, source_vector) in source_vectors.iter().take(sample_len).enumerate() {
        let norm = source_vector_l2_norm(source_vector);
        if !norm.is_finite() {
            return Err(format!(
                "ec_diskann {context} requires finite unit-normalized source vectors; sampled non-finite ||v|| at position {idx}"
            ));
        }
        norm_sum += norm;
        min_norm = min_norm.min(norm);
        max_norm = max_norm.max(norm);
        if !(low..=high).contains(&norm) && first_outlier.is_none() {
            first_outlier = Some((idx, norm));
        }
    }

    let mean_norm = norm_sum / sample_len as f32;
    if !(low..=high).contains(&mean_norm) || first_outlier.is_some() {
        let outlier_suffix = first_outlier
            .map(|(idx, norm)| format!("; first outlier at sample {idx} had ||v|| = {norm:.4}"))
            .unwrap_or_default();
        return Err(format!(
            "ec_diskann {context} requires unit-normalized source vectors for the v0 distance wrapper; sampled {sample_len} vector(s) with mean ||v|| = {mean_norm:.4} and range [{min_norm:.4}, {max_norm:.4}], expected within [{low:.4}, {high:.4}]{outlier_suffix}"
        ));
    }

    Ok(())
}

#[cfg(any(test, feature = "bench"))]
pub(crate) fn source_inner_product_scalar_reference(left: &[f32], right: &[f32]) -> f32 {
    ambuild::source_inner_product_scalar_reference(left, right)
}

#[cfg(all(any(test, feature = "bench"), target_arch = "x86_64"))]
pub(crate) fn source_inner_product_avx2_fma_for_test(left: &[f32], right: &[f32]) -> Option<f32> {
    ambuild::source_inner_product_avx2_fma_for_test(left, right)
}

#[cfg(all(any(test, feature = "bench"), target_arch = "aarch64"))]
pub(crate) fn source_inner_product_neon_for_test(left: &[f32], right: &[f32]) -> Option<f32> {
    ambuild::source_inner_product_neon_for_test(left, right)
}

pub(super) fn warn_on_non_unit_source_vector(source_vector: &[f32], context: &str) {
    if let Err(message) = validate_source_vector_unit_norm(source_vector, context) {
        emit_unit_norm_warning(&message);
    }
}

pub(super) fn warn_on_non_unit_source_vector_sample(
    source_vectors: &[&[f32]],
    sample_cap: usize,
    context: &str,
) {
    if let Err(message) =
        validate_source_vector_unit_norm_sample(source_vectors, sample_cap, context)
    {
        emit_unit_norm_warning(&message);
    }
}

#[inline]
pub(super) fn maybe_check_for_interrupts() {
    #[cfg(test)]
    {}

    #[cfg(not(test))]
    {
        pgrx::check_for_interrupts!();
    }
}

fn emit_unit_norm_warning(message: &str) {
    #[cfg(all(test, not(feature = "pg_test")))]
    {
        let _ = message;
    }

    #[cfg(not(all(test, not(feature = "pg_test"))))]
    {
        pgrx::warning!("{message}");
    }
}

fn source_vector_l2_norm(source_vector: &[f32]) -> f32 {
    source_vector
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::{
        validate_source_vector_unit_norm, validate_source_vector_unit_norm_sample,
        ECDISKANN_UNIT_NORM_BUILD_SAMPLE_CAP,
    };

    #[test]
    fn validate_source_vector_unit_norm_accepts_unit_vectors() {
        assert!(
            validate_source_vector_unit_norm(&[0.6, 0.8], "unit test").is_ok(),
            "vectors with ||v|| = 1 should satisfy the v0 DiskANN precondition",
        );
    }

    #[test]
    fn validate_source_vector_unit_norm_rejects_non_unit_vectors() {
        let error = validate_source_vector_unit_norm(&[2.0, 0.0], "unit test")
            .expect_err("non-unit vectors must be rejected");
        assert!(error.contains("unit-normalized"));
        assert!(error.contains("||v|| = 2.0000"));
    }

    #[test]
    fn validate_source_vector_unit_norm_sample_reports_sample_stats() {
        let unit = [1.0_f32, 0.0];
        let non_unit = [2.0_f32, 0.0];
        let sample = vec![unit.as_slice(), non_unit.as_slice()];
        let error = validate_source_vector_unit_norm_sample(
            &sample,
            ECDISKANN_UNIT_NORM_BUILD_SAMPLE_CAP,
            "ambuild",
        )
        .expect_err("non-unit build samples must be rejected");
        assert!(error.contains("sampled 2 vector(s)"));
        assert!(error.contains("first outlier"));
    }
}
