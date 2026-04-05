# Review: score_code_inner_product Allocates Per Call

**File:** `src/lib.rs:119-136`
**Severity:** Medium (performance in hot path)
**Category:** Optimization

## Finding

```rust
pub(crate) fn score_code_inner_product(
    dim: usize, bits: u8, seed: u64,
    code_a: &[u8], code_b: &[u8],
) -> f32 {
    let quantizer = ProdQuantizer::cached(dim, bits, seed);
    let mut payload_a = Vec::with_capacity(MIN_BINARY_BYTES + code_a.len());
    payload_a.extend_from_slice(&0.0_f32.to_le_bytes());
    payload_a.extend_from_slice(code_a);

    let mut payload_b = Vec::with_capacity(MIN_BINARY_BYTES + code_b.len());
    payload_b.extend_from_slice(&0.0_f32.to_le_bytes());
    payload_b.extend_from_slice(code_b);

    quantizer.score_ip_encoded_lite(&payload_a, &payload_b)
}
```

This function:
1. Acquires the cache mutex lock (via `cached`)
2. Allocates two `Vec<u8>` payloads (~776 bytes each for 1536-dim 4-bit)
3. Copies the code bytes into them with a 4-byte gamma prefix

This is called during:
- `tqvector_inner_product` (SQL function, per-row in queries)
- `build_scored_neighbor_graph` (O(n^2) calls during build)
- `entry_point_score` (per-neighbor per-candidate during build)
- `BuildCodeDistance::eval` (per distance evaluation during HNSW construction)

For the HNSW build with N vectors and M neighbors, this means O(N * M * ef_construction) allocations, each creating two ~800-byte vectors.

## Recommendation

`score_ip_encoded_lite` only needs the MSE portion of the payload. It calls `split_payload` which just slices into the byte array. The 4-byte gamma prefix is parsed but ignored. Consider:

1. **Add a `score_ip_codes_lite` method** that takes raw code bytes directly (no gamma prefix needed), avoiding the allocation entirely.
2. **Or** use a stack-allocated buffer: for 1536-dim 4-bit, the payload is 772 bytes, which could be stack-allocated with a small-vec or arrayvec.

Option 1 is cleaner:
```rust
pub fn score_codes_lite(&self, code_a: &[u8], code_b: &[u8]) -> f32 {
    let mut mse_sum = 0.0_f32;
    for dim_index in 0..self.original_dim {
        let idx_a = mse_index_at(code_a, dim_index, self.bits - 1) as usize;
        let idx_b = mse_index_at(code_b, dim_index, self.bits - 1) as usize;
        mse_sum += self.codebook[idx_a] * self.codebook[idx_b];
    }
    mse_sum
}
```

## Action Required

Add a zero-allocation code-to-code scoring path for the build hot path.
