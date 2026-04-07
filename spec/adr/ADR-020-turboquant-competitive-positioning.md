---
id: ADR-020
title: "TurboQuant Competitive Positioning in the PostgreSQL Vector Quantization Landscape"
status: DECIDED
impact: Affects FR-013, FR-015, ADR-006, NFR-002, NFR-003
date: 2026-04-06
---
# ADR-020: TurboQuant Competitive Positioning in the PostgreSQL Vector Quantization Landscape

## Context

tqvector implements TurboQuant (ICLR 2026) as its quantization layer. This ADR documents the competitive landscape of vector quantization approaches in PostgreSQL extensions and vector databases, and why TurboQuant occupies a unique position.

### The TurboQuant Algorithm

TurboQuant is a two-stage, data-oblivious quantization method:

1. **Stage 1 (MSE)**: Apply SRHT (Subsampled Randomized Hadamard Transform) rotation, then optimal Lloyd-Max scalar quantizers per dimension. The rotation concentrates coordinates to a Beta distribution, making all inputs equally quantization-friendly. No training data needed — codebooks are fully determined by `(dim, bits)`.

2. **Stage 2 (QJL)**: Apply a 1-bit Quantized Johnson-Lindenstrauss transform to the residual between the original and stage-1 reconstruction. This provides an **unbiased** inner product estimator with provably near-optimal distortion.

Key properties:
- **Data-oblivious**: No codebook training, no k-means, no data-dependent parameters
- **Online**: New vectors can be compressed immediately — no warm-up period
- **Near-optimal**: Achieves the information-theoretic distortion rate bound
- **Unbiased**: QJL correction ensures the inner product estimator has zero systematic error

### Quantization Methods in the PostgreSQL Ecosystem

| Extension | Method | Compression (1536-dim) | Training | Recall@10 | IP Estimator |
|---|---|---|---|---|---|
| **pgvector** | halfvec (fp16) | 2x | None | ~100% (minimal loss) | Exact on fp16 |
| **pgvector** | binary_quantize | 32x | None | Poor (<70% without reranking) | Hamming distance |
| **pgvectorscale** | Statistical Binary Quantization | ~32x | None | Claimed SOTA for binary | Hamming + statistical correction |
| **VectorChord** | RaBitQ (1-bit) | 32x | IVF clustering | High (formal guarantees) | Randomized binary estimator |
| **VectorChord** | RaBitQ (4-bit) | 8x | IVF clustering | Very high | Multi-bit RaBitQ |
| **VectorChord** | RaBitQ (8-bit) | 4x | IVF clustering | ~99% | 8-bit randomized binary |
| **Lantern** | Product Quantization | Up to 97% reduction | k-means per subspace | Moderate (degrades at high compression) | Codebook lookup |
| **Weaviate** (not PG) | 8-bit Rotational Quantization | 4x | None | 98-99% | Walsh-Hadamard + scalar |
| **tqvector** | TurboQuant (MSE+QJL) | **7.7x** at 4-bit | **None** | **~97%** (NFR-003 target) | **Unbiased LUT-based** |

### Key Differentiators

#### vs Product Quantization (pgvector IVFFlat, Lantern)

| Property | TurboQuant | Product Quantization |
|---|---|---|
| Training | None (data-oblivious) | k-means on subspaces (minutes to hours) |
| Recall at aggressive compression | Superior at every bit-width (ICLR paper) | Degrades badly — top-10 overlap <10% at heavy PQ |
| New data | Compress immediately | Must fit existing codebook or retrain |
| Codebook storage | Zero (deterministic from dim, bits) | Stored per-index |
| Distribution sensitivity | None (rotation normalizes) | Degrades on non-clustered data |
| Scoring | Zero-allocation LUT, O(d) | Codebook lookup, O(d/m × 2^b) |

The ICLR paper demonstrates TurboQuant outperforming PQ on recall at every bit-width tested, while reducing indexing time to virtually zero.

#### vs RaBitQ (VectorChord)

RaBitQ is the closest competitor — both use randomization for formal guarantees:

| Property | TurboQuant | RaBitQ |
|---|---|---|
| Core technique | SRHT rotation + Lloyd-Max + QJL residual | Random rotation + binary rounding |
| Bit-width flexibility | Configurable 1-8 bits per dimension | 1, 4, or 8 bits |
| Compression at 4-bit | ~7.7x (783 bytes at 1536-dim) | ~8x |
| Unbiased estimator | Yes (QJL correction) | Yes (randomized rounding) |
| Index structure | HNSW (topology-agnostic) | IVF (requires clustering) |
| Training | None | IVF centroid training |
| Formal guarantees | Near-optimal distortion rate | Asymptotically optimal space-accuracy |

RaBitQ's formal guarantees are stronger in theory, but it's tied to IVF's centroid-based clustering. TurboQuant's data-oblivious design pairs with any index structure.

#### vs Rotational Quantization (Weaviate)

Weaviate's 8-bit RQ is mathematically equivalent to TurboQuant's MSE stage without the QJL correction:

| Component | Weaviate 8-bit RQ | TurboQuant |
|---|---|---|
| Rotation | Walsh-Hadamard (3 rounds, 256-block) | SRHT (full-dimension) |
| Quantization | 8-bit scalar per dimension | b-bit Lloyd-Max per dimension |
| Residual correction | **None** | **QJL 1-bit random projection** |
| Recall (1M vectors) | 98-99% at 4x compression | ~97% at **7.7x** compression |
| Build time | ~7us/vector rotation | Similar (SRHT is O(d log d)) |

The QJL stage is what enables TurboQuant to maintain high recall at 4-bit (7.7x compression) rather than requiring 8-bit (4x compression).

#### vs Binary Quantization (pgvector, pgvectorscale)

Binary quantization achieves maximum compression (32x) but recall is poor without reranking against full-precision vectors. It works best on high-dimensional embeddings (1024+) where each bit still carries signal. TurboQuant at 4-bit achieves 7.7x compression with ~97% recall — a much better tradeoff for most workloads.

### Scoring Performance

tqvector's LUT-based scoring is a key architectural advantage:

1. **Query preparation** (once per query): Rotate query via SRHT, compile into a lookup table indexed by `(dimension_group, quantized_value)`. Cost: O(d log d) for rotation + O(d × 2^b) for LUT construction.

2. **Per-candidate scoring** (hot loop): Table lookups + integer accumulation + QJL bit-expansion correction. Cost: O(d) with **zero heap allocation**. Measured: ~95K scores/sec at 1536-dim 4-bit (BC-010).

PQ scoring requires codebook lookups with similar O(d) cost but larger lookup tables (per-subspace codebooks). TurboQuant's LUT is ~48KB for 1536-dim 4-bit (FR-017-AC-4), fitting comfortably in L1 cache.

### What No One Else Has

**No existing PostgreSQL extension implements TurboQuant.** The competitive landscape:

| System | TurboQuant Status |
|---|---|
| pgvector | No quantization beyond halfvec/binary |
| pgvectorscale | SBQ only |
| VectorChord | RaBitQ only |
| Lantern | PQ only |
| Qdrant | Requested (issue #8524), not implemented |
| Milvus | No TurboQuant support |
| Weaviate | RQ (equivalent to TurboQuant Stage 1 only) |
| **tqvector** | **Full implementation (MSE + QJL + SIMD)** |

## Decision

**TurboQuant is the correct quantization choice for tqvector.** It uniquely combines:
1. Data-oblivious compression (works on any distribution — agent memories, knowledge graphs)
2. Unbiased inner product estimation (QJL correction)
3. Near-optimal distortion rate (formal guarantee)
4. Zero-training, zero-codebook, zero-warmup operation
5. Configurable bit-width (1-8 bits per dimension)
6. LUT-based scoring with zero allocation per call

No competing approach offers all six properties simultaneously.

## Consequences

### Positive
- First PostgreSQL extension with TurboQuant — unique market position
- Data-oblivious design matches tqvector's multi-use-case architecture (ADR-017)
- No codebook training means immediate indexing of new data
- QJL correction maintains recall at aggressive compression (4-bit, 7.7x)

### Negative
- Cannot exploit cluster structure the way PQ/RaBitQ can — but the rotation makes this unnecessary
- ICLR 2026 paper is recent — limited production deployment data outside tqvector
- No community-maintained crate — tqvector must maintain its own implementation (ADR-006)
- SIMD implementations (AVX2, NEON) must be maintained for two architectures

### Neutral
- Weaviate's adoption of the same core technique (rotational quantization) validates the mathematical foundation
- Qdrant's community request for TurboQuant validates market interest
- The quantization layer is independent of the index structure — TurboQuant works with HNSW today and could work with IVF or DiskANN in the future

## References

### TurboQuant
- [TurboQuant: Online Vector Quantization with Near-optimal Distortion Rate (ICLR 2026)](https://arxiv.org/abs/2504.19874) — original paper
- [OpenReview: TurboQuant](https://openreview.net/forum?id=tO3ASKZlok) — peer review and discussion
- [Google Research: TurboQuant blog](https://research.google/blog/turboquant-redefining-ai-efficiency-with-extreme-compression/) — accessible overview
- [HuggingFace: TurboQuant paper page](https://huggingface.co/papers/2504.19874) — community discussion

### Competing Quantization Methods
- [RaBitQ: Quantizing High-Dimensional Vectors with a Theoretical Error Bound (SIGMOD 2024)](https://dl.acm.org/doi/pdf/10.1145/3654970) — formal guarantees, asymptotically optimal
- [RaBitQ paper (arXiv)](https://arxiv.org/pdf/2405.12497) — full paper with proofs
- [Weaviate: 8-bit Rotational Quantization](https://weaviate.io/blog/8-bit-rotational-quantization) — same mathematical foundation as TurboQuant Stage 1
- [Pinecone: Product Quantization](https://www.pinecone.io/learn/series/faiss/product-quantization/) — PQ overview, 97% compression at cost of recall
- [HuggingFace: Binary and Scalar Embedding Quantization](https://huggingface.co/blog/embedding-quantization) — practical comparison of SQ, BQ for embeddings
- [Zilliz: Scalar Quantization and Product Quantization](https://zilliz.com/learn/scalar-quantization-and-product-quantization) — fundamentals

### PostgreSQL Vector Extension Landscape
- [VectorChord 1.0 blog](https://blog.vectorchord.ai/vectorchord-10-developer-first-vector-search-on-postgres-100x-faster-indexing-than-pgvector) — IVF+RaBitQ positioning
- [VectorChord: Store 400k Vectors for $1](https://blog.vectorchord.ai/vectorchord-store-400k-vectors-for-1-in-postgresql) — cost-efficiency claims
- [pgvectorscale GitHub](https://github.com/timescale/pgvectorscale) — StreamingDiskANN + SBQ
- [Lantern: Product Quantization](https://lantern.dev/blog/pq) — PQ in PostgreSQL
- [Lantern: pgvector Storage Internals](https://lantern.dev/blog/pgvector-storage) — HNSW page layout analysis

### Industry Adoption
- [Qdrant issue #8524: TurboQuant support request](https://github.com/qdrant/qdrant/issues/8524) — 10 thumbs-up, community Rust pseudocode
- [Search Engine Land: Google TurboQuant algorithm improves vector search speed](https://searchengineland.com/google-turboquant-algorithm-vector-search-472977) — industry coverage
- [Vizuara: TurboQuant explainer](https://vizuara.substack.com/p/turboquant-online-vector-quantization) — technical deep dive

### Related Academic Work
- [QJL: 1-Bit Quantized JL Transform (AAAI 2025)](https://arxiv.org/abs/2502.00527) — the residual correction stage
- [Quantization Meets Projection (arXiv:2411.06158)](https://arxiv.org/pdf/2411.06158) — theoretical analysis of rotation + quantization
