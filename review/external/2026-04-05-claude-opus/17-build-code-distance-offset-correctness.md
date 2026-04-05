# Review: BuildCodeDistance Score Offset Correctness

**File:** `src/am/mod.rs:1258-1281`
**Severity:** Medium (affects graph quality)
**Category:** Correctness

## Finding

`BuildCodeDistance` converts inner product similarity to a distance for `hnsw_rs`:

```rust
impl BuildCodeDistance {
    fn new(dimensions: usize, bits: u8, seed: u64) -> Self {
        let quantizer = crate::quant::prod::ProdQuantizer::cached(dimensions, bits, seed);
        let max_abs_centroid = quantizer.codebook.iter()
            .map(|value| value.abs())
            .fold(0.0_f32, f32::max);
        Self {
            score_offset: dimensions as f32 * max_abs_centroid * max_abs_centroid,
            // ...
        }
    }
}

impl Distance<u8> for BuildCodeDistance {
    fn eval(&self, va: &[u8], vb: &[u8]) -> f32 {
        self.score_offset - score_code_inner_product(...)
    }
}
```

The offset is `dim * max_centroid^2`. This is the theoretical maximum of the MSE-only inner product (achieved when all coordinates of both vectors are quantized to the max-absolute centroid with the same sign).

**Concern:** The `score_ip_encoded_lite` function computes `sum(codebook[idx_a] * codebook[idx_b])`. For the maximum to be `dim * max_centroid^2`, both vectors must quantize to the same index at every dimension, AND all centroids must have the same absolute value. Since Lloyd-Max centroids are not all equal magnitude, the actual maximum inner product could be less than `dim * max_centroid^2`.

However, the offset only needs to be an **upper bound** to ensure all distances are non-negative (which `hnsw_rs` requires). So this is correct -- it's just not a tight bound. A looser bound means the distance values are shifted higher than necessary, but since `hnsw_rs` only uses relative ordering of distances, this doesn't affect graph quality.

**Verified: Correct.** The offset is a valid upper bound. The distance function preserves the correct ordering of neighbors.

## One edge case

If `max_abs_centroid` is 0 (all centroids are zero, which would mean a degenerate codebook), then `score_offset = 0` and all distances would be `0 - 0 = 0`, making all vectors equidistant. This can't happen with a valid Lloyd-Max codebook for bits >= 2.

## Action Required

None. Code is correct.
