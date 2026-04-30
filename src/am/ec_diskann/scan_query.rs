//! Query-side scan primitives for `ec_diskann` (task 17 Phase 6B-1).
//!
//! Pure-Rust helpers that sit between a persisted Vamana index and the
//! pgrx scan callbacks. Nothing here depends on pgrx; everything is
//! driven by `&DataPageChain`, `VamanaMetadataPage`, and the raw query
//! vector. The Phase 6B-2 pgrx layer will stitch these into
//! `amrescan` / `amgettuple`.
//!
//! Three primitives:
//!
//! 1. [`read_grouped_codebook_chain`] — walk a persisted codebook
//!    shard chain (head TID → `nexttid` terminator) and concatenate
//!    the shards' centroids into a single flat `Vec<f32>` of shape
//!    `[group_count][GROUPED_PQ_CENTROIDS * group_size]`. Verifies the
//!    shards arrive in `group_index` order.
//! 2. [`encode_query_srht`] — apply the metadata's SRHT forward
//!    transform to a raw query vector. Seed + dimensions are read from
//!    the metadata page so scan-time and build-time agree on the
//!    rotation.
//! 3. [`build_grouped_pq_lut_from_persisted`] — convenience that ties
//!    the above into a single call, returning the `[group][centroid]`
//!    LUT that `grouped_pq_score_f32` consumes.
//!
//! The shapes here match Phase 5C-3's build-time pipeline so
//! scan-time scoring agrees with the codes that ambuild wrote.

use crate::am::common::training::SrhtForwardTransform;
use crate::am::ec_diskann::tuple::VamanaCodebookTuple;
use crate::quant::grouped_pq::{build_grouped_pq_lut_f32, GROUPED_PQ_CENTROIDS};
use crate::storage::page::{DataPageChain, ItemPointer};

/// Walk a persisted codebook chain starting at `head_tid` and return
/// the concatenated flat codebook matrix (row-major, group-major).
///
/// Output layout matches [`build_grouped_pq_lut_f32`]'s
/// `flat_codebooks` parameter: `group_count * GROUPED_PQ_CENTROIDS *
/// group_size` floats, with group 0 first, group 1 next, etc. Callers
/// must pass the `centroid_count = GROUPED_PQ_CENTROIDS * group_size`
/// — this matches the `centroid_count` argument to
/// [`VamanaCodebookTuple::decode`].
///
/// Validates that:
///
/// - The head TID is not `INVALID`.
/// - Each shard's `group_index` equals its position in the walk.
/// - The last shard's `nexttid` is `INVALID`.
/// - Exactly `group_count` shards are reached.
pub fn read_grouped_codebook_chain(
    chain: &DataPageChain,
    head_tid: ItemPointer,
    group_count: usize,
    centroid_count: usize,
) -> Result<Vec<f32>, String> {
    if group_count == 0 {
        return Err("codebook chain requires group_count >= 1".into());
    }
    if head_tid == ItemPointer::INVALID {
        return Err("codebook chain head TID is INVALID".into());
    }

    let mut flat = Vec::with_capacity(group_count * centroid_count);
    let mut cursor = head_tid;

    for expected_group in 0..group_count {
        if cursor == ItemPointer::INVALID {
            return Err(format!(
                "codebook chain terminated early at group {expected_group} of {group_count}"
            ));
        }
        let page = chain
            .get_page(cursor.block_number)
            .ok_or_else(|| format!("codebook page {} not in chain", cursor.block_number))?;
        let raw = page.raw_tuple(cursor)?;
        let tuple = VamanaCodebookTuple::decode(raw, centroid_count)?;
        if tuple.group_index as usize != expected_group {
            return Err(format!(
                "codebook shard out of order: expected group_index {expected_group}, got {}",
                tuple.group_index
            ));
        }
        if tuple.centroids.len() != centroid_count {
            return Err(format!(
                "codebook shard {expected_group} centroid count mismatch: got {}, expected {centroid_count}",
                tuple.centroids.len()
            ));
        }
        flat.extend_from_slice(&tuple.centroids);
        cursor = tuple.nexttid;
    }

    if cursor != ItemPointer::INVALID {
        return Err(format!(
            "codebook chain longer than declared group_count {group_count}"
        ));
    }

    Ok(flat)
}

/// Apply the metadata-page SRHT forward transform to a raw query
/// vector, returning the rotated vector (length =
/// `rotation::effective_transform_dim(dimensions)`). The rotated
/// output is what [`build_grouped_pq_lut_f32`] expects as its query
/// input. Seed + dimensions match the values baked into the metadata
/// page at ambuild.
pub fn encode_query_srht(raw_query: &[f32], dimensions: usize, seed: u64) -> Vec<f32> {
    let transform = SrhtForwardTransform::for_dimensions(dimensions, seed);
    transform.apply(raw_query)
}

/// Pack sign bits from a scan-time rotated query into the same word layout as
/// the persisted binary sidecar. Bit `i % 64` of word `i / 64` is set when
/// rotated coordinate `i` is non-negative.
pub fn pack_query_sign_bits(rotated_query: &[f32], dimensions: usize) -> Vec<u64> {
    let mut words = vec![0_u64; dimensions.div_ceil(64)];
    for (index, value) in rotated_query.iter().copied().take(dimensions).enumerate() {
        if value >= 0.0 {
            words[index / 64] |= 1_u64 << (index % 64);
        }
    }
    words
}

/// Hamming distance between scan-time query sign words and a persisted binary
/// sidecar. Smaller values are better and can be used directly as a Vamana
/// prefilter score.
pub fn hamming_xor_popcount(query_words: &[u64], candidate_words: &[u64]) -> u32 {
    query_words
        .iter()
        .zip(candidate_words.iter())
        .map(|(query, candidate)| (query ^ candidate).count_ones())
        .sum()
}

/// One-shot helper: load the codebooks from `chain` at `codebook_head`,
/// SRHT-encode the query, and build the scoring LUT in one call.
///
/// Returns `(lut, group_count)` so the caller can feed
/// [`crate::quant::grouped_pq::grouped_pq_score_f32`] directly.
pub fn build_grouped_pq_lut_from_persisted(
    chain: &DataPageChain,
    codebook_head: ItemPointer,
    group_count: usize,
    group_size: usize,
    dimensions: usize,
    seed: u64,
    raw_query: &[f32],
) -> Result<(Vec<f32>, usize), String> {
    if group_size == 0 {
        return Err("group_size must be >= 1".into());
    }
    let centroid_count = GROUPED_PQ_CENTROIDS * group_size;
    let flat = read_grouped_codebook_chain(chain, codebook_head, group_count, centroid_count)?;
    let rotated = encode_query_srht(raw_query, dimensions, seed);
    if rotated.len() != group_count * group_size {
        return Err(format!(
            "rotated query length {} does not match group_count {group_count} * group_size {group_size}",
            rotated.len()
        ));
    }
    let lut = build_grouped_pq_lut_f32(&rotated, &flat, group_size);
    Ok((lut, group_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::common::training::{train_grouped_pq4_model, GroupedPq4Model};
    use crate::am::ec_diskann::persist::stage_grouped_codebook_chain;
    use crate::storage::page::DEFAULT_PAGE_SIZE;

    fn synth_model(group_count: usize, group_size: usize) -> GroupedPq4Model {
        let centroid_count = GROUPED_PQ_CENTROIDS * group_size;
        let codebooks = (0..group_count)
            .map(|g| {
                (0..centroid_count)
                    .map(|i| (g * 1000 + i) as f32 * 0.125)
                    .collect()
            })
            .collect();
        GroupedPq4Model {
            codebooks,
            group_count,
            group_size,
            transform_dim: group_size * group_count,
            signs: vec![1.0; group_size * group_count],
        }
    }

    // CR-001: round-trip — stage a 4-group codebook, read it back, the
    // flat output equals concat(codebooks[0], codebooks[1], ...).
    #[test]
    fn cr_001_multi_group_roundtrip_preserves_order() {
        let group_count = 4;
        let group_size = 8;
        let centroid_count = GROUPED_PQ_CENTROIDS * group_size;
        let model = synth_model(group_count, group_size);
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let head = stage_grouped_codebook_chain(&mut chain, &model).expect("stage");

        let flat =
            read_grouped_codebook_chain(&chain, head, group_count, centroid_count).expect("read");
        assert_eq!(flat.len(), group_count * centroid_count);

        let mut expected = Vec::new();
        for g in 0..group_count {
            expected.extend_from_slice(&model.codebooks[g]);
        }
        assert_eq!(flat, expected);
    }

    #[test]
    fn cr_009_pack_query_sign_bits_matches_manual_word_layout() {
        let rotated = [-1.0, 0.0, 0.25, -0.5, 1.0];
        let words = pack_query_sign_bits(&rotated, rotated.len());
        assert_eq!(words, vec![0b10110]);
    }

    #[test]
    fn cr_010_hamming_xor_popcount_scores_word_pairs() {
        let query = [0b1010_u64, 0b1111];
        let candidate = [0b0011_u64, 0b0101];
        assert_eq!(hamming_xor_popcount(&query, &candidate), 4);
    }

    // CR-002: single-group codebook reads back cleanly.
    #[test]
    fn cr_002_single_group_roundtrip() {
        let group_size = 4;
        let centroid_count = GROUPED_PQ_CENTROIDS * group_size;
        let model = synth_model(1, group_size);
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let head = stage_grouped_codebook_chain(&mut chain, &model).expect("stage");
        let flat = read_grouped_codebook_chain(&chain, head, 1, centroid_count).expect("read");
        assert_eq!(flat, model.codebooks[0]);
    }

    // CR-003: INVALID head TID is rejected.
    #[test]
    fn cr_003_invalid_head_rejected() {
        let chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let err = read_grouped_codebook_chain(&chain, ItemPointer::INVALID, 1, 64)
            .expect_err("should fail");
        assert!(err.contains("INVALID"), "got: {err}");
    }

    // CR-004: declaring more groups than the chain carries is detected.
    #[test]
    fn cr_004_declared_group_count_too_high_errors() {
        let group_size = 4;
        let centroid_count = GROUPED_PQ_CENTROIDS * group_size;
        let model = synth_model(2, group_size);
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let head = stage_grouped_codebook_chain(&mut chain, &model).expect("stage");
        let err =
            read_grouped_codebook_chain(&chain, head, 3, centroid_count).expect_err("should fail");
        assert!(err.contains("terminated early"), "got: {err}");
    }

    // CR-005: declaring fewer groups than the chain carries is detected.
    #[test]
    fn cr_005_declared_group_count_too_low_errors() {
        let group_size = 4;
        let centroid_count = GROUPED_PQ_CENTROIDS * group_size;
        let model = synth_model(3, group_size);
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let head = stage_grouped_codebook_chain(&mut chain, &model).expect("stage");
        let err =
            read_grouped_codebook_chain(&chain, head, 2, centroid_count).expect_err("should fail");
        assert!(err.contains("longer than declared"), "got: {err}");
    }

    // CR-006: mismatched centroid_count surfaces the length error from
    // VamanaCodebookTuple::decode.
    #[test]
    fn cr_006_centroid_count_mismatch_errors() {
        let group_size = 4;
        let model = synth_model(2, group_size);
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let head = stage_grouped_codebook_chain(&mut chain, &model).expect("stage");
        // Caller lies about centroid count.
        let err = read_grouped_codebook_chain(&chain, head, 2, 7).expect_err("should fail");
        assert!(err.contains("length mismatch"), "got: {err}");
    }

    // CR-010: SRHT encoding agrees with the build-time transform.
    // Encoding the same query twice is deterministic, and the output
    // length matches effective_transform_dim(dimensions).
    #[test]
    fn cr_010_srht_encode_is_deterministic_and_sized() {
        use crate::quant::rotation::effective_transform_dim;
        let dims = 13;
        let seed = 42;
        let q: Vec<f32> = (0..dims).map(|i| (i as f32) * 0.25).collect();
        let a = encode_query_srht(&q, dims, seed);
        let b = encode_query_srht(&q, dims, seed);
        assert_eq!(a, b);
        assert_eq!(a.len(), effective_transform_dim(dims));
    }

    // CR-011: end-to-end — train a grouped-PQ4 model on random vectors,
    // stage the codebook, then scan-time rebuild of the LUT from the
    // persisted chain produces byte-identical output to a directly-
    // computed LUT over the in-memory model.
    #[test]
    fn cr_011_end_to_end_lut_matches_in_memory() {
        use crate::quant::grouped_pq::build_grouped_pq_lut_f32;
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let dims = 32;
        let seed = 17;
        let group_size = 8;
        let n_train = 256;
        let mut rng = ChaCha8Rng::seed_from_u64(5);
        let training: Vec<Vec<f32>> = (0..n_train)
            .map(|_| (0..dims).map(|_| rng.gen::<f32>()).collect())
            .collect();
        let training_refs: Vec<&[f32]> = training.iter().map(|v| v.as_slice()).collect();
        let model =
            train_grouped_pq4_model(&training_refs, dims, seed, group_size, 128, 4).expect("train");

        // Stage the codebook into a DataPageChain.
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let head = stage_grouped_codebook_chain(&mut chain, &model).expect("stage");

        let query: Vec<f32> = (0..dims).map(|_| rng.gen::<f32>()).collect();

        // Scan-time path.
        let (lut_scan, g) = build_grouped_pq_lut_from_persisted(
            &chain,
            head,
            model.group_count,
            model.group_size,
            dims,
            seed,
            &query,
        )
        .expect("scan-time lut");
        assert_eq!(g, model.group_count);

        // In-memory reference.
        let rotated = encode_query_srht(&query, dims, seed);
        let flat: Vec<f32> = model.codebooks.iter().flatten().copied().collect();
        let lut_ref = build_grouped_pq_lut_f32(&rotated, &flat, model.group_size);

        assert_eq!(lut_scan, lut_ref);
    }

    // CR-012: group_size = 0 is rejected before any chain read.
    #[test]
    fn cr_012_zero_group_size_rejected() {
        let chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let err = build_grouped_pq_lut_from_persisted(
            &chain,
            ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            1,
            0,
            16,
            42,
            &[0.0; 16],
        )
        .expect_err("should fail");
        assert!(err.contains("group_size"), "got: {err}");
    }
}
