# Architecture

## Overview

Ecaz is a PostgreSQL extension (pgrx) with three main subsystems:

1. **Quantizer core** — TurboQuant compression (MSE + QJL two-stage quantization)
2. **HNSW index** — graph-based approximate nearest neighbor index access method
3. **SQL interface** — type system, operators, and bootstrap

## Compression Pipeline

```
fp32 vector (1536 floats, 6144 bytes)
  |
  v
SRHT rotation (randomized Hadamard transform)
  |
  v
MSE stage: Lloyd-Max codebook quantization (4-bit)
  |
  v
QJL stage: Gaussian projection residual correction
  |
  v
tqvector datum (783 bytes)
```

The quantizer produces unbiased inner product estimates — the expected value of the compressed distance equals the true fp32 distance.

## Index Layout

The `ec_hnsw` index uses a page layout modeled on pgvector's approach:

- **Element tuples** — store the compressed vector code and a heap TID pointer
- **Neighbor tuples** — store the HNSW graph adjacency lists
- Standard PostgreSQL 8KB pages with WAL support

Graph construction and runtime traversal both use Ecaz-owned HNSW primitives,
with the build output materialized into PostgreSQL pages for runtime traversal.

## SIMD Acceleration

Performance-critical paths have SIMD implementations:

- **FWHT** (Fast Walsh-Hadamard Transform) — AVX2+FMA / NEON / scalar
- **Scoring** — vectorized inner product over quantized codes
- **Bit operations** — bitpacking for MSE index decode

Runtime CPU detection selects the best available path.

## Key Design Decisions

See [Architecture Decision Records](../spec/adr/) for detailed rationale on design choices including:

- [ADR-001](../spec/adr/ADR-001-code-to-code-scoring.md) — Code-to-code scoring
- [ADR-006](../spec/adr/ADR-006-own-quantizer.md) — Own quantizer (in-tree)
- [ADR-004](../spec/adr/ADR-004-pgrx-index-am.md) — pgrx index AM approach
- [ADR-017](../spec/adr/ADR-017-hnsw-over-ivf.md) — HNSW over IVF
- [ADR-019](../spec/adr/ADR-019-wal-write-amplification.md) — WAL write amplification

## Further Reading

- [Specification](../spec/spec.md) — full requirements specification
- [TurboQuant paper](https://arxiv.org/abs/2504.19874)
