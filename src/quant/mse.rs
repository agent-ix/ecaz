//! Scalar MSE-stage helpers.

use crate::quant::CodeIndex;

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
std::thread_local! {
    static NEAREST_CENTROID_INDEX_16_CALLS: Cell<usize> = const { Cell::new(0) };
}

pub fn nearest_centroid_index(codebook: &[f32], value: f32) -> CodeIndex {
    // Branchless update: rather than `if distance < best { ... }` we
    // compute `is_better` once per iteration and use it to blend
    // `best_distance` and `best_index`. The compiler reliably lowers
    // the trailing-`if` blend into `cmov` / `select`. Tie-breaking
    // (lower index wins on equal distance) is preserved by the strict
    // `<` comparison.
    let mut best_index = 0_u16;
    let mut best_distance = f32::INFINITY;
    for (index, centroid) in codebook.iter().enumerate() {
        let distance = (value - *centroid).abs();
        let is_better = distance < best_distance;
        best_distance = if is_better { distance } else { best_distance };
        best_index = if is_better { index as u16 } else { best_index };
    }
    best_index
}

/// Fully-unrolled 16-centroid scan for the production `(1536, 4)` path.
///
/// Takes a `&[f32; 16]` (not a slice) so the compiler can lift bounds
/// checks and unroll the loop completely. Same branchless tie-break
/// rule as `nearest_centroid_index`.
//
// `needless_range_loop` would rewrite this as `iter().enumerate()`,
// which obscures the constant trip count from the optimizer and
// blocks full loop unrolling. The whole point of this function is the
// constant `1..16` range — keep it explicit.
#[allow(clippy::needless_range_loop)]
pub fn nearest_centroid_index_16(codebook: &[f32; 16], value: f32) -> CodeIndex {
    #[cfg(test)]
    NEAREST_CENTROID_INDEX_16_CALLS.with(|count| count.set(count.get() + 1));

    let mut best_index = 0_u16;
    let mut best_distance = (value - codebook[0]).abs();
    for index in 1..16_usize {
        let distance = (value - codebook[index]).abs();
        let is_better = distance < best_distance;
        best_distance = if is_better { distance } else { best_distance };
        best_index = if is_better { index as u16 } else { best_index };
    }
    best_index
}

pub fn quantize_to_indices(codebook: &[f32], rotated: &[f32], dim: usize) -> Vec<CodeIndex> {
    if let Ok(codebook_16) = <&[f32; 16]>::try_from(codebook) {
        return rotated[..dim]
            .iter()
            .map(|value| nearest_centroid_index_16(codebook_16, *value))
            .collect();
    }
    rotated[..dim]
        .iter()
        .map(|value| nearest_centroid_index(codebook, *value))
        .collect()
}

pub fn decode_indices(codebook: &[f32], indices: &[CodeIndex]) -> Vec<f32> {
    indices
        .iter()
        .map(|index| codebook[*index as usize])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::hint::black_box;
    use std::time::Instant;

    /// The pre-optimization branching scan, kept around in tests only
    /// as the bit-exact reference for the branchless rewrite.
    fn nearest_centroid_index_branching(codebook: &[f32], value: f32) -> CodeIndex {
        let mut best_index = 0usize;
        let mut best_distance = f32::INFINITY;
        for (index, centroid) in codebook.iter().enumerate() {
            let distance = (value - *centroid).abs();
            if distance < best_distance {
                best_distance = distance;
                best_index = index;
            }
        }
        best_index as CodeIndex
    }

    #[test]
    fn nearest_centroid_index_prefers_lower_index_on_tie() {
        let codebook = [-1.0_f32, 0.0, 1.0];
        assert_eq!(nearest_centroid_index(&codebook, 0.5), 1);
    }

    #[test]
    fn branchless_matches_branching_over_random_inputs() {
        // Cross-check the branchless scan against the original
        // branching scan over a wide grid of (codebook, value) pairs.
        // Codebook sizes 4, 8, 16, 32, 64, 128 cover every production
        // bit width 2..=7.
        let mut rng = ChaCha8Rng::seed_from_u64(0xDEADBEEF);
        for &num_centroids in &[4_usize, 8, 16, 32, 64, 128] {
            let codebook: Vec<f32> = (0..num_centroids)
                .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
                .collect();
            for _ in 0..2000 {
                let value: f32 = rng.gen_range(-1.5_f32..1.5_f32);
                let branchless = nearest_centroid_index(&codebook, value);
                let branching = nearest_centroid_index_branching(&codebook, value);
                assert_eq!(
                    branchless, branching,
                    "branchless diverged from branching at \
                     num_centroids={num_centroids}, value={value}"
                );
            }
        }
    }

    #[test]
    fn unrolled_16_matches_generic_branchless() {
        // The 16-centroid fast path must produce exactly the same
        // index as the generic branchless scan for every (codebook,
        // value) pair. This is the path quantize_to_indices dispatches
        // to on the (1536, 4) production case.
        let mut rng = ChaCha8Rng::seed_from_u64(0xC0DECAFE);
        for _ in 0..200 {
            let codebook_vec: Vec<f32> = (0..16)
                .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
                .collect();
            let codebook_16: [f32; 16] = codebook_vec
                .as_slice()
                .try_into()
                .expect("16-element codebook");
            for _ in 0..200 {
                let value: f32 = rng.gen_range(-1.5_f32..1.5_f32);
                let unrolled = nearest_centroid_index_16(&codebook_16, value);
                let generic = nearest_centroid_index(&codebook_vec, value);
                assert_eq!(
                    unrolled, generic,
                    "unrolled-16 diverged from generic branchless at value={value}"
                );
            }
        }
    }

    #[test]
    fn quantize_to_indices_matches_per_element_scan_for_16_centroids() {
        // Output-equivalence check: the quantize_to_indices 16-centroid
        // dispatch must produce the same indices as the explicit generic
        // branchless per-element scan would.
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let codebook: Vec<f32> = (0..16)
            .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
            .collect();
        let rotated: Vec<f32> = (0..1536)
            .map(|_| rng.gen_range(-1.5_f32..1.5_f32))
            .collect();
        let dispatched = quantize_to_indices(&codebook, &rotated, 1536);
        let manual: Vec<CodeIndex> = rotated
            .iter()
            .map(|v| nearest_centroid_index(&codebook, *v))
            .collect();
        assert_eq!(dispatched, manual);
    }

    #[test]
    fn quantize_to_indices_dispatches_unrolled_for_16_centroids() {
        let mut rng = ChaCha8Rng::seed_from_u64(8);
        let codebook: Vec<f32> = (0..16)
            .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
            .collect();
        let rotated: Vec<f32> = (0..1536)
            .map(|_| rng.gen_range(-1.5_f32..1.5_f32))
            .collect();

        NEAREST_CENTROID_INDEX_16_CALLS.with(|count| count.set(0));
        let _ = quantize_to_indices(&codebook, &rotated, 1536);
        let dispatch_calls = NEAREST_CENTROID_INDEX_16_CALLS.with(Cell::get);
        assert_eq!(
            dispatch_calls,
            1536,
            "16-centroid dispatch should call nearest_centroid_index_16 once per input value"
        );
    }

    #[test]
    // Measured on Linux 6.17.0-19-generic (x86_64), Rust stable, on
    // 2026-04-10 in `--release`: branching = 1.74s, generic = 1.75s,
    // unrolled = 1.59s, unrolled-vs-branching speedup = 1.10x,
    // unrolled-vs-generic = 1.10x.
    #[ignore = "microbenchmark; run manually with --ignored --nocapture"]
    fn nearest_centroid_index_16_microbench() {
        let mut rng = ChaCha8Rng::seed_from_u64(0x1234_5678);
        let codebook_vec: Vec<f32> = (0..16)
            .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
            .collect();
        let codebook_16: [f32; 16] = codebook_vec
            .as_slice()
            .try_into()
            .expect("16-element codebook");
        let rotated: Vec<f32> = (0..1536)
            .map(|_| rng.gen_range(-1.5_f32..1.5_f32))
            .collect();
        let iterations = 100_000;

        let mut branching_sum = 0_usize;
        let branching_start = Instant::now();
        for _ in 0..iterations {
            for value in &rotated {
                branching_sum +=
                    nearest_centroid_index_branching(black_box(&codebook_vec), black_box(*value))
                        as usize;
            }
        }
        let branching_elapsed = branching_start.elapsed();

        let mut generic_sum = 0_usize;
        let generic_start = Instant::now();
        for _ in 0..iterations {
            for value in &rotated {
                generic_sum +=
                    nearest_centroid_index(black_box(&codebook_vec), black_box(*value)) as usize;
            }
        }
        let generic_elapsed = generic_start.elapsed();

        let mut unrolled_sum = 0_usize;
        let unrolled_start = Instant::now();
        for _ in 0..iterations {
            for value in &rotated {
                unrolled_sum +=
                    nearest_centroid_index_16(black_box(&codebook_16), black_box(*value)) as usize;
            }
        }
        let unrolled_elapsed = unrolled_start.elapsed();

        assert_eq!(branching_sum, generic_sum);
        assert_eq!(generic_sum, unrolled_sum);

        let branching_speedup = branching_elapsed.as_secs_f64() / unrolled_elapsed.as_secs_f64();
        let generic_speedup = generic_elapsed.as_secs_f64() / unrolled_elapsed.as_secs_f64();
        eprintln!(
            "nearest_centroid_index_16_microbench branching={branching_elapsed:?} generic={generic_elapsed:?} unrolled={unrolled_elapsed:?} unrolled_vs_branching={branching_speedup:.2}x unrolled_vs_generic={generic_speedup:.2}x"
        );
    }
}
