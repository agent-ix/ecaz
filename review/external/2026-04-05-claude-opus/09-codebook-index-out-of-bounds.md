# Review: MSE Codebook Index Bounds Not Validated at Decode

**File:** `src/quant/mse.rs:25-30`, `src/quant/prod.rs:159-167`
**Severity:** Medium (potential panic / UB under corrupt data)
**Category:** Correctness / safety

## Finding

`decode_indices` indexes into the codebook without bounds checking:

```rust
pub fn decode_indices(codebook: &[f32], indices: &[CodeIndex]) -> Vec<f32> {
    indices
        .iter()
        .map(|index| codebook[*index as usize])  // panics if index >= codebook.len()
        .collect()
}
```

Similarly, `score_ip_encoded` indexes into the LUT:

```rust
let centroid_index = mse_index_at(mse_packed, dim_index, self.bits - 1) as usize;
mse_sum += prepared.lut[dim_index * num_centroids + centroid_index];
```

If a corrupt or malicious `tqvector` datum contains packed MSE indices that decode to values >= `2^(bits-1)`, this will panic (index out of bounds). In a PostgreSQL extension, a panic in Rust code is caught by pgrx's panic handler and converted to a PostgreSQL ERROR, so it won't crash the backend. However:

1. The error message will be unhelpful ("index out of bounds")
2. Scoring during query execution should not panic on data corruption -- it should produce a reasonable error or skip the tuple
3. The `read_bits_le` function can return values with more bits set than expected if the packed buffer has trailing garbage bits

## Recommendation

Add validation either:
1. In `mse_index_at`: mask the result to `(1 << bits_per_index) - 1`
2. Or at decode time: check `centroid_index < codebook.len()` and produce a descriptive error

The masking approach is simpler and zero-cost:

```rust
fn mse_index_at(packed: &[u8], dim_index: usize, bits_per_index: u8) -> CodeIndex {
    read_bits_le(packed, dim_index * bits_per_index as usize, bits_per_index as usize)
        & ((1 << bits_per_index) - 1)  // already guaranteed by bit width, but defensive
}
```

Actually, `read_bits_le` already only reads `width` bits, so the value is naturally bounded. The real risk is if `bits_per_index` is wrong or if the packed buffer is truncated. Both are validated upstream. This finding is **lower risk than initially assessed** but worth a bounds check in the scoring hot path for defense-in-depth.

## Action Required

Consider adding a debug_assert on `centroid_index < num_centroids` in `score_ip_encoded` and `score_ip_encoded_lite` to catch corruption during development without runtime cost in release.
