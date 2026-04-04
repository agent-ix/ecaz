---
id: FR-015
title: ProdQuantizer Orchestrator
type: functional-requirement
status: APPROVED
object_type: entity
traces:
  - FR-013
  - FR-005
  - FR-017
  - FR-014
---
# FR-015: ProdQuantizer Orchestrator

## Requirement

The extension SHALL implement a `ProdQuantizer` struct in `quant/prod.rs` that orchestrates the two-stage quantization pipeline and exposes the complete encode/decode/score API used by all other components.

### Struct Definition

```rust
pub struct ProdQuantizer {
    transform_dim: usize, // padded dimension used by FWHT workspace
    original_dim: usize,  // original persisted dimensionality
    bits: u8,             // quantization bits (2–8)
    seed: u64,            // PRNG seed for rotation + QJL projection
    codebook: Vec<f64>,   // Lloyd-Max centroids, length = 2^(bits-1)
    signs: Vec<f32>,      // diagonal sign vector for SRHT, length = transform_dim
}
```

The ProdQuantizer is **fully determined by `(original_dim, bits, seed)`** — construction requires no training data. All internal state (codebook, signs, PRNG state for QJL) is derived deterministically from these three parameters.

### Construction

```rust
impl ProdQuantizer {
    pub fn new(dim: usize, bits: u8, seed: u64) -> Self;
}
```

1. Derive `transform_dim` as the next power of two >= `dim`
2. Generate Lloyd-Max codebook via `codebook::lloyd_max(bits - 1, original_dim, 20_000)` (FR-013)
3. Generate diagonal sign vector from `seed` via ChaCha20 PRNG
4. Store all state — the quantizer is ready to encode immediately
5. The implementation SHALL cache immutable `ProdQuantizer` instances per backend, keyed by `(original_dim, bits, seed)`, so repeated queries and inserts do not regenerate codebooks or sign vectors

### Encode API

```rust
pub struct EncodedTq {
    pub gamma: f32,
    pub mse_packed: Vec<u8>,
    pub qjl_packed: Vec<u8>,
}

pub fn encode(&self, vector: &[f32]) -> EncodedTq
```

1. Pad input to `transform_dim` in scratch space (zero-pad if shorter)
2. Apply SRHT rotation: diagonal signs × FWHT × scale
3. Quantize the first `original_dim` rotated coordinates to nearest codebook centroids → MSE indices
4. Reconstruct the MSE-only approximation in the original input domain with zero-filled transform tail
5. Compute the residual norm `gamma = ||x - x_tilde_mse||_2`
6. Project the residual through the QJL SRHT projection → QJL signs
7. Bit-pack MSE indices for the original dimensionality into `ceil(original_dim * (bits-1) / 8)` bytes
8. Bit-pack QJL signs for the original dimensionality into `ceil(original_dim / 8)` bytes
9. Return `{ gamma, mse_packed, qjl_packed }`

### Decode API (Approximate)

```rust
pub fn decode_approximate(&self, codes: &[u8]) -> Vec<f32>
```

1. Unpack MSE indices from code bytes
2. Map each index to its codebook centroid value
3. Zero-fill the discarded transform tail `[original_dim, transform_dim)`
4. Apply inverse SRHT rotation (inverse FWHT × inverse diagonal signs)
5. Truncate to `original_dim`
6. Return the MSE-only approximate f32 vector (the QJL residual is not reconstructed by this API)

### LUT Preparation

```rust
pub struct PreparedQuery {
    pub lut: Vec<f32>,
    pub sq: Vec<f32>,
    pub qjl_scale: f32,
}

pub fn prepare_ip_query(&self, query: &[f32]) -> PreparedQuery
```

1. Rotate the raw query into the MSE transform domain
2. For each original dimension `i` and each centroid `c`:
   `lut[i * num_centroids + c] = codebook[c] * query_rotated[i]`
3. Return a `PreparedQuery` containing the flat LUT array of shape `[original_dim × num_centroids]`
4. Project the raw query through the QJL transform and retain the first `original_dim` projected coordinates as `sq`
5. Store `qjl_scale = sqrt(pi / 2) / original_dim`

Memory layout: contiguous `f32` array, row-major `[original_dim][num_centroids]`. This layout is cache-friendly for the scoring loop which iterates dimension-by-dimension.

### Scoring API

#### LUT-based scoring (query already prepared)

```rust
pub fn score_ip_encoded(&self, prepared: &PreparedQuery, candidate_codes: &[u8]) -> f32
```

This API is the implementation vehicle for the prepared-query scoring contract defined in FR-017 and the estimator formula defined in FR-013.

1. Unpack candidate MSE indices
2. For each dimension `i`: accumulate `prepared.lut[i * num_centroids + candidate_idx[i]]`
3. Unpack candidate `gamma`
4. Compute `qjl_sum = Σ prepared.sq[i] * sign(candidate_qjl[i])`
5. Compute `qjl_correction = gamma * prepared.qjl_scale * qjl_sum`
6. Return `mse_sum + qjl_correction`

**Zero heap allocation.** SIMD-accelerated (FR-014).

#### Code-to-code scoring (no LUT)

```rust
pub fn score_ip_encoded_lite(&self, codes_a: &[u8], codes_b: &[u8]) -> f32
```

1. Unpack MSE indices from both codes
2. For each dimension `i`: accumulate `codebook[idx_a[i]] * codebook[idx_b[i]]`
3. SHALL ignore the QJL payload and gamma in v0.1
4. Return `mse_sum`

Used during aminsert page-level beam search. SIMD-accelerated (FR-014).

### Pack / Unpack Utilities

```rust
pub fn pack_mse_indices(indices: &[CodeIndex], bits_per_index: u8) -> Vec<u8>
pub fn unpack_mse_indices(packed: &[u8], dim: usize, bits_per_index: u8) -> Vec<CodeIndex>
```

Bit-packing for MSE indices at arbitrary bit widths (1–7 bits per index). Indices are packed sequentially in little-endian bit order.

```rust
pub fn pack_qjl_signs(signs: &[bool]) -> Vec<u8>
pub fn unpack_qjl_signs(packed: &[u8], dim: usize) -> Vec<bool>
```

1-bit-per-sign packing, little-endian byte order.

### Thread Safety

`ProdQuantizer` is `Send + Sync` — all state is immutable after construction. Multiple scoring calls may execute concurrently (relevant for parallel sequential scan in Postgres 14+).

## Acceptance Criteria

### FR-015-AC-1: Encode produces correct code length
`ProdQuantizer::new(1536, 4, 42).encode(&vec)` SHALL produce a quantized payload of exactly 772 bytes, consisting of 4-byte `gamma`, 576-byte `mse_packed`, and 192-byte `qjl_packed`.

### FR-015-AC-2: Encode + decode round-trip
Decode(encode(v)) SHALL have cosine similarity > 0.85 with v on average over a fixed-seed sample of 100 random 1536-dim unit vectors.

### FR-015-AC-3: Prepared-query scoring matches the declared formula
`score_ip_encoded` with a prepared query SHALL match the formula declared in FR-013 within floating-point tolerance.

### FR-015-AC-4: Code-to-code scoring is symmetric
`score_ip_encoded_lite(a, b) == score_ip_encoded_lite(b, a)` for all valid inputs.

### FR-015-AC-5: Pack/unpack round-trip
pack then unpack of MSE indices SHALL be lossless for all bit widths 1–7.

### FR-015-AC-6: Deterministic construction
Two `ProdQuantizer::new(1536, 4, 42)` instances SHALL produce identical codebooks and sign vectors.

### FR-015-AC-7: Zero allocation in score_ip_encoded
Repeated calls to `score_ip_encoded` with the same LUT SHALL not allocate heap memory (verified by benchmarking).

### FR-015-AC-8: Backend-local cache reuse
Repeated construction requests for the same `(original_dim, bits, seed)` within a backend SHALL reuse cached immutable quantizer state.

### FR-015-AC-9: Code-to-code scorer ignores QJL in v0.1
Altering only `gamma` and QJL bits while keeping MSE indices fixed SHALL NOT change `score_ip_encoded_lite`.
