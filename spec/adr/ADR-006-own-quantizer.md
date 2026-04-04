---
id: ADR-006
title: "Reimplement quantizer from TurboQuantDB instead of using turbo-quant crate"
status: DECIDED
impact: Resolves ADR-001, ADR-005. Affects all FRs.
date: 2026-04-04
---
# ADR-006: Own quantizer implementation based on TurboQuantDB

## Context

The `turbo-quant` crate (v0.1) was the original plan. Investigation revealed three problems:

1. **Storage size**: PolarQuant stores f32 radii → ~5,017 bytes per code at 1536-dim 4-bit.
   TurboQuantDB's MSE+QJL stores bit-packed integer codes → 768 bytes. 6.5x difference.

2. **Scoring performance**: `turbo-quant` regenerates the full m×d Gaussian projection matrix
   (2.25 MB at 1536-dim) on every `inner_product_estimate` call. TurboQuantDB uses pre-computed
   lookup tables and scores with zero allocation per call.

3. **No SIMD**: `turbo-quant` is scalar-only. TurboQuantDB has AVX2+FMA for FWHT, MSE scoring,
   and QJL bit-expansion.

## Decision

**Drop `turbo-quant` crate.** Extract the quantizer core from `~/dev_bak/TurboQuantDB/` and adapt for pgrx.

### What to extract

```
TurboQuantDB/src/quantizer/
  codebook.rs    → tqvector/src/quant/codebook.rs
  mse.rs         → tqvector/src/quant/mse.rs
  qjl.rs         → tqvector/src/quant/qjl.rs
  prod.rs        → tqvector/src/quant/prod.rs

TurboQuantDB/src/linalg/
  hadamard.rs    → tqvector/src/quant/hadamard.rs
  rotation.rs    → tqvector/src/quant/rotation.rs
```

~1,200 lines of quantizer + linalg code.

### Adaptations required

| Change | Reason |
|---|---|
| Replace `ndarray::Array1<f64>` with `&[f32]` / `Vec<f32>` | Remove ndarray dependency, match Postgres float4 |
| Remove rayon | Postgres backends are single-threaded per query |
| Remove storage layer (WAL, segments, live_codes) | Postgres owns storage |
| Add aarch64 NEON SIMD alongside AVX2 | ARM64 server support (AWS Graviton, etc.) |
| Allocate LUT in Postgres memory context where possible | Auto-cleanup on transaction end |

### SIMD targets

| Function | x86_64 AVX2+FMA | aarch64 NEON |
|---|---|---|
| `fwht` (Walsh-Hadamard) | Existing | New |
| `score_ip_encoded` (LUT-based) | Existing | New |
| `score_ip_encoded_lite` (code-to-code) | Existing | New |

Runtime feature detection with scalar fallback on both architectures.

### What this resolves

- **ADR-001** (code-to-code): TurboQuantDB has `prepare_ip_query_from_codes` + `score_ip_encoded_lite` — code-to-code with zero decode
- **ADR-005** (serialization): MSE codes are already bit-packed integers. Storage format is `pack_mse_indices` (576 bytes) + QJL bits (192 bytes) = 768 bytes. No conversion layer needed.
