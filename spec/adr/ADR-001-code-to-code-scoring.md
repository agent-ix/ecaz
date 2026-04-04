---
id: ADR-001
title: "turbo-quant crate lacks code-to-code inner product"
status: DECIDED
impact: HIGH for FR-005, FR-008, FR-009 (HNSW AM)
date: 2026-04-03
---
# ADR-001: turbo-quant crate lacks code-to-code inner product

## Context

The `turbo-quant` crate (v0.1) exposes:

```rust
TurboQuantizer::inner_product_estimate(&self, code: &TurboCode, query: &[f32]) -> Result<f32>
```

This is **asymmetric**: one side is a compressed code, the other is a raw f32 query vector. There is no `inner_product_estimate(&TurboCode, &TurboCode)`.

## Investigation Results

### TurboCode has public fields + serde

```rust
pub struct TurboCode {
    pub polar_code: PolarCode,     // public: radii (Vec<f32>), angle_indices (Vec<u16>), dim, bits
    pub residual_sketch: QjlSketch, // public: signs (Vec<i8>), dim, projections
}
```

Both `TurboCode` and its sub-types derive `Serialize, Deserialize` (serde) and `Clone`. All fields are public. We can serialize to bincode/bytes for storage.

### decode_approximate exists

```rust
TurboQuantizer::decode_approximate(&self, code: &TurboCode) -> Result<Vec<f32>>
// Only reconstructs PolarQuant. QJL sketch is for IP correction, not reconstruction.
```

### Feasible workaround: asymmetric with decode

For HNSW queries (search path), the asymmetric API is actually the right one:
- The **query** is always a raw f32 vector (user provides it)
- The **database vectors** are stored as TurboCode
- `inner_product_estimate(db_code, raw_query)` is exactly what we need for scan

For HNSW **build** (graph construction), we need to compare code-to-code (two database vectors). Here we use `decode_approximate` on one side:
- Decode one code to f32 (~O(d) work, one allocation per call)
- Call `inner_product_estimate(other_code, decoded_f32)`

For HNSW **insert** (single vector), the new vector is f32 (from the INSERT statement) — asymmetric works directly.

## Decision

**Option D: Asymmetric + decode for build only.** This is actually not as bad as feared:

1. **Search (hot path)**: query is always f32 → asymmetric API is native, zero overhead
2. **Insert**: new vector is f32 → asymmetric API is native
3. **Build**: decode one side per edge comparison. Build is a one-time bulk operation, not latency-sensitive. Acceptable.
4. **Vacuum repair**: same as build — one-time graph repair, not latency-sensitive

No fork or upstream contribution needed for v0.1.

## Future Optimization

If build time becomes a bottleneck at scale, contribute a `code_to_code_inner_product` function upstream that operates directly on `PolarCode` fields (radii + angle_indices) without full decode. The math is straightforward:

```
⟨x, y⟩ ≈ Σᵢ rₓᵢ · rᵧᵢ · cos(θₓᵢ - θᵧᵢ) + QJL_correction
```

This can be computed directly from the stored codes without decode.
