---
id: ADR-021
title: "Default Vector Dimension: 2048 over 1536, with 1024 and 1536 as Supported Tiers"
status: PROPOSED
impact: Affects FR-013, FR-004, NFR-002, NFR-003, ADR-006, ADR-020
date: 2026-04-08
---
# ADR-021: Default Vector Dimension — 2048 over 1536, with 1024 and 1536 as Supported Tiers

## Context

tqvector's quantization pipeline relies on the Fast Walsh-Hadamard Transform (FWHT), which **strictly requires power-of-2 input lengths** (`hadamard.rs:7-11`). Non-power-of-2 dimensions are zero-padded to the next power of two via `rotation::transform_dim()` before any rotation occurs. This creates a structural disadvantage for 1536-dimensional vectors — the most common embedding dimension today — because they must be padded to 2048 internally.

This ADR evaluates 1024, 1536, and 2048 as target dimensions, proposes 2048 as the recommended default, and defines an architecture for supporting all three as first-class tiers.

### The Zero-Padding Problem at 1536

When a 1536-dim vector enters the quantization pipeline (`prod.rs:88-90`):

```
1. pad_input(vector, 2048)      → 1536 signal dims + 512 zero dims
2. srht(&padded, &signs)        → 2048 rotated dims (energy spread uniformly)
3. quantize_to_indices(..., 1536) → keep only first 1536 of 2048 rotated dims
```

The SRHT is an orthonormal transform — it preserves the input norm but **spreads energy uniformly** across all 2048 output dimensions. By keeping only 1536 of 2048 rotated coordinates, we discard 1536/2048 = **25% of the rotated signal energy**. This has three consequences:

1. **Increased MSE**: The stage-1 reconstruction captures less of the original vector's information. The residual norm (gamma) is larger.
2. **Harder QJL correction**: A larger residual means the 1-bit QJL stage must correct a bigger error, reducing its effectiveness.
3. **Codebook distribution mismatch**: The Lloyd-Max codebook is generated with `d=1536` (`prod.rs:52`), parameterizing the post-rotation Beta PDF as Beta(768, 768). But the actual rotation occurs in 2048-dim space, where the true marginal is closer to Beta(1024, 1024). This is a subtle mismatch that increases quantization distortion.

For power-of-2 dimensions (1024, 2048), `original_dim == transform_dim`. No padding, no energy loss, no codebook mismatch.

### Computational Cost Parity: 1536 Already Pays 2048 Prices

Because 1536 pads to 2048 internally, the FWHT and sign-vector costs are **identical** to native 2048:

| Resource | 1024 | 1536 (padded → 2048) | 2048 |
|---|---|---|---|
| FWHT butterfly ops | 10,240 | **22,528** | **22,528** |
| Sign vector memory | 8 KB | **16 KB** | **16 KB** |
| `pad_input` allocation | 0 | 8 KB (2048 × f32) | 0 |

1536-dim vectors pay the full 2048-dim FWHT cost but retain only 75% of the resulting signal. Moving to native 2048 costs nothing extra at the rotation stage while gaining 33% more signal dimensions.

### Storage and Scoring Costs

Where dimensions diverge is in per-vector storage and per-candidate scoring, both of which scale linearly with `original_dim`:

**Storage at 4-bit quantization:**

| Dimension | MSE code | QJL code | Payload | Datum (11+payload) | Element tuple |
|---|---|---|---|---|---|
| 1024 | 384 B | 128 B | 516 B | 527 B | 586 B |
| 1536 | 576 B | 192 B | 772 B | 783 B | 842 B |
| 2048 | 768 B | 256 B | 1,028 B | 1,039 B | 1,098 B |

**Derived metrics at 4-bit:**

| Metric | 1024 | 1536 | 2048 |
|---|---|---|---|
| Compression vs fp32 | 7.77x | 7.85x | 7.88x |
| Vectors per 8 KB page | 13 | 9 | 7 |
| LUT size (8 centroids) | 32 KB | 48 KB | 64 KB |
| L1 cache fit (32-64 KB) | Comfortable | Fits | Borderline |
| Scoring throughput (est.) | ~143K/s | ~95K/s | ~71K/s |
| Energy retained after SRHT | 100% | **75%** | 100% |

The 2048 tier trades 33% more storage and ~25% lower scoring throughput for full energy retention and zero padding waste. The 1024 tier is 33% cheaper than 1536 with the same FWHT alignment benefit.

### Tiled FWHT: Mitigating Non-Power-of-2 Dimensions

A tiled (block-diagonal) FWHT decomposes the full-dimension transform into independent power-of-2 blocks, eliminating the need for zero-padding:

| Dimension | Tiling | Tiles | Pad per tile | Cross-tile decorrelation |
|---|---|---|---|---|
| 1024 | 1 × 1024 | 1 | 0 | Full (single block) |
| 1536 | 3 × 512 | 3 | 0 | Within 512-dim tiles only |
| 2048 | 1 × 2048 or 2 × 1024 | 1–2 | 0 | Full or within 1024-dim tiles |

Tiled FWHT for 1536 eliminates zero-padding and restores 100% energy retention, at the cost of reduced decorrelation scope. Weaviate uses a similar approach: 3-round, 256-block tiled rotation (ADR-020, line 79). However:

- Smaller tiles mean coordinates are decorrelated only within their tile, not globally. The theoretical guarantee of the SRHT concentrating all coordinates to a single Beta distribution weakens.
- The codebook is still generated assuming full-dimension rotation. Tile-local rotation produces a different per-coordinate distribution than full-dimension rotation.
- For 2048, a single full-dimension FWHT is optimal and naturally aligned. Tiling is only needed if cache pressure demands it (for future higher dimensions like 3072 or 4096).

**Tiled FWHT is valuable for keeping 1536 viable as a compatibility tier, but does not eliminate the structural advantage of power-of-2 native dimensions.**

### Embedding Model Landscape

| Dimension | Models |
|---|---|
| 768 | Google text-embedding-004, BERT-based models |
| 1024 | Cohere embed-v3, Jina v3 (default), many open-source |
| 1536 | OpenAI text-embedding-3-small, ada-002 (legacy) |
| 2048 | OpenAI text-embedding-3-large (Matryoshka truncation) |
| 3072 | OpenAI text-embedding-3-large (native) |

Modern models increasingly support [Matryoshka Representation Learning](https://arxiv.org/abs/2205.13147), allowing truncation to arbitrary dimensions with graceful quality degradation. OpenAI's text-embedding-3-large can be truncated to 2048 with minimal recall loss versus its native 3072. This makes 2048 a practical choice even when the source model is higher-dimensional.

### Hardware Evaluation: AWS Graviton on RDS

The primary deployment target for tqvector is PostgreSQL on Amazon RDS, which runs on AWS Graviton processors (ARM Neoverse cores). The L1 data cache is the binding constraint for scoring throughput.

**Graviton cache hierarchy:**

| Generation | Core | L1D | L2 | L3 (shared) | SIMD |
|---|---|---|---|---|---|
| Graviton2 | Neoverse N1 | 64 KB | 1 MB | 32 MB | NEON (128-bit) |
| Graviton3 | Neoverse V1 | 64 KB | 1 MB | 32 MB | SVE (256-bit) |
| Graviton4 | Neoverse V2 | 64 KB | 2 MB | 36 MB | SVE2 (128-bit) |

**All Graviton generations have 64 KB L1D.** L1 hit latency is ~4 cycles; L2 load-to-use is ~10 cycles (2.5x penalty per miss).

#### Scoring hot working set

The per-candidate scoring loop (`prod.rs:184-199`) touches three data structures:

| Component | Role | Access pattern |
|---|---|---|
| **LUT** | `dim × 2^(bits-1) × f32` | Stride by num_centroids, reused across all candidates |
| **sq vector** | `dim × f32` | Sequential, reused across all candidates |
| **Candidate payload** | MSE + QJL packed bytes | Sequential, changes per candidate |

**Working set sizes at 4-bit (8 centroids):**

| Component | 1024 | 1536 | 2048 |
|---|---|---|---|
| LUT | 32 KB | 48 KB | **64 KB** |
| sq vector | 4 KB | 6 KB | 8 KB |
| Candidate payload | 512 B | 768 B | 1 KB |
| **Total hot set** | **36.5 KB** | **54.8 KB** | **73 KB** |
| **L1D utilization** | 57% | 86% | **114% — spills** |

At 2048-dim 4-bit, the LUT alone fills the entire 64 KB L1D. The sq vector and candidate data are forced into L2, incurring 2.5x latency on every access. The hardware prefetcher partially mitigates this (access is sequential), but the throughput impact is measurable.

**Working set sizes at 3-bit (4 centroids):**

| Component | 1024 | 1536 | 2048 |
|---|---|---|---|
| LUT | 16 KB | 24 KB | 32 KB |
| sq vector | 4 KB | 6 KB | 8 KB |
| Candidate payload | 384 B | 576 B | 768 B |
| **Total hot set** | **20.4 KB** | **30.6 KB** | **40.8 KB** |
| **L1D utilization** | 32% | 48% | 64% — fits |

At 3-bit, all three dimension tiers fit comfortably in L1D.

**Working set sizes at 5-bit (16 centroids):**

| Component | 1024 | 1536 | 2048 |
|---|---|---|---|
| LUT | 64 KB | 96 KB | 128 KB |
| **L1D utilization** | **100% — borderline** | 150% — spills | 200% — badly spills |

At 5+ bits, even 1024 starts to pressure L1D.

#### Key insight: 2048@3bit is payload-equivalent to 1536@4bit

A striking configuration emerges when combining dimension and bit-width:

| Configuration | MSE code | QJL code | Payload | LUT | L1D fit | Total info bits | FWHT aligned |
|---|---|---|---|---|---|---|---|
| 1536 @ 4-bit | 576 B | 192 B | **772 B** | 48 KB | Tight | 6,144 | No (25% pad waste) |
| 2048 @ 3-bit | 512 B | 256 B | **772 B** | 32 KB | Comfortable | 6,144 | **Yes** |

**Identical payload size. Identical total information bits. But 2048@3bit has:**
- Zero FWHT padding waste (power-of-2 aligned)
- 100% energy retention vs 75%
- 32 KB LUT vs 48 KB — better L1D headroom (50% vs 86% utilization)
- Codebook perfectly matches post-rotation distribution
- 33% more dimensions of signal compensating for fewer centroids per dimension

The trade-off is 4 centroids per dimension (3-bit) vs 8 centroids (4-bit). Fewer centroids increase per-dimension quantization error, but the 33% increase in signal dimensions and elimination of padding waste compensate. **This configuration needs empirical recall validation but is theoretically favorable.**

#### Page I/O and shared_buffers on RDS

HNSW traversal follows graph edges across pages. Page density affects I/O efficiency:

| Dimension | Element tuple | Tuples per 8 KB page | 1M vectors index size |
|---|---|---|---|
| 1024 | 586 B | 13 | ~542 MB |
| 1536 | 842 B | 9 | ~811 MB |
| 2048 | 1,098 B | 7 | ~1,080 MB |

Typical RDS Graviton instance shared_buffers:

| Instance | RAM | shared_buffers (est.) | 1M vectors fit? | 10M vectors fit? |
|---|---|---|---|---|
| db.r7g.large | 16 GB | ~4 GB | All tiers | 1024 only |
| db.r7g.xlarge | 32 GB | ~8 GB | All tiers | 1024, 1536 |
| db.r7g.2xlarge | 64 GB | ~16 GB | All tiers | All tiers |

Higher page density (1024-dim: 13/page vs 2048-dim: 7/page) means fewer page reads per graph traversal — especially important when the index exceeds shared_buffers and hits disk.

#### SIMD considerations

The MSE LUT scoring loop has data-dependent indexing (`centroid_index` varies per dimension), limiting SIMD vectorization. The QJL accumulation loop is fully sequential and SIMD-friendly. Graviton3's 256-bit SVE can process 8 f32s per cycle in the QJL loop, while Graviton2/4 use 128-bit NEON/SVE2 (4 f32s per cycle). This makes the QJL loop ~2x faster on Graviton3, but since it's roughly half the scoring work, the net benefit is ~30%.

## Decision

### 1. Recommend 2048 as the default dimension, 3-bit as the Graviton-optimal bit-width

2048 is the optimal dimension for tqvector's FWHT-based pipeline:
- Zero padding waste (power-of-2 aligned)
- 100% rotated energy retained
- Codebook exactly matches post-rotation distribution
- Identical FWHT cost to what 1536 already pays
- Compatible with Matryoshka-truncated embeddings from 3072+ models

**For RDS Graviton deployments, 2048@3bit is the recommended configuration.** It delivers the same payload size and total information bits as 1536@4bit while fitting the 32 KB LUT comfortably in Graviton's 64 KB L1D (50% utilization vs 86%). The 2048@4bit configuration spills L1D and should be reserved for non-latency-sensitive batch workloads or hardware with larger L1 caches.

### 2. Define three supported dimension tiers

| Tier | Dimension | Recommended bits | Use Case | FWHT | LUT in L1D |
|---|---|---|---|---|---|
| **Quality** (default) | 2048 | 3-bit (Graviton) / 4-bit (x86) | Maximum recall, Matryoshka large models | Full-dim, no padding | 32 KB (3b) / 64 KB (4b) |
| **Compact** | 1024 | 4-bit | Cost-sensitive, smaller models (Cohere, Jina) | Full-dim, no padding | 32 KB |
| **Compatibility** | 1536 | 4-bit | OpenAI legacy, migration from pgvector | Tiled FWHT (3×512) or padded | 48 KB |

All three are first-class citizens. tqvector already accepts any dimension 1–65535 at the type level — the tiers are **recommended configurations** with optimized benchmarks and documentation, not hard constraints.

### 3. Implement tiled FWHT for the 1536 compatibility tier

To keep 1536 competitive:
- Implement block-diagonal FWHT with configurable tile size
- Default tiling: factor `original_dim` into power-of-2 tiles (1536 = 3 × 512)
- Generate per-tile sign vectors (independent seeds per tile)
- Adapt codebook parameterization to use tile dimension, not full dimension
- Fall back to padded full-dim FWHT when `original_dim` is already power-of-2

### 4. Fix codebook parameterization for non-power-of-2 dimensions

Currently `lloyd_max` receives `original_dim` (`prod.rs:52`). For padded FWHT mode, it should receive `transform_dim` to match the actual post-rotation marginal distribution. For tiled FWHT mode, it should receive the tile dimension. This eliminates the codebook-distribution mismatch.

## Architecture Changes

### Already dimension-agnostic (no changes needed)

- `ProdQuantizer::new(dim, bits, seed)` — parametric on any dimension
- Index metadata stores `dimensions: u16` per index (`page.rs:65`)
- Scan validates query dim against index dim (`scan.rs:82-90`)
- Payload encoding/decoding is fully parametric on `original_dim`
- `CodeIndex = u16` supports up to 65,536 centroids — unchanged

### Changes required

| Component | Change | Scope |
|---|---|---|
| `quant/hadamard.rs` | Add `fwht_tiled_in_place(values, tile_size)` | New function |
| `quant/rotation.rs` | Add `tiled_srht(input, signs, tile_size)` that applies per-tile FWHT | New function |
| `quant/rotation.rs` | `transform_dim()` unchanged for padded mode; new `tile_dims()` for tiled mode | New function |
| `quant/prod.rs` | `ProdQuantizer` gains optional `tile_size` field; encode/decode branch on tiling strategy | Modify struct + `new()` + `encode()` |
| `quant/codebook.rs` | `lloyd_max` receives effective rotation dimension (transform_dim or tile_dim) | Parameter semantics change |
| `tests/size_of_assertions.rs` | Add size assertions for 1024-dim and 2048-dim at 4-bit | New tests |
| `benches/` | Add criterion benchmarks for all three tiers | New benchmarks |
| `sql/bootstrap.sql` | Document recommended dimension tiers in function comments | Documentation |

### Wire format compatibility

The datum wire format (`[dim:u16][bits:u8][seed:u64][gamma:f32][codes:...]`) is **unchanged**. The tiling strategy is deterministic from `(dim, seed)` — readers reconstruct the same tiling at decode time. No new fields are needed in the datum or index metadata.

However, changing the codebook parameterization (fix #4) changes the codebook centroids for non-power-of-2 dimensions. This means **encoded vectors at 1536-dim with the old codebook are not compatible** with the corrected codebook. Since tqvector has not shipped a stable release, this is acceptable. If a migration path were needed, the seed could encode the codebook strategy.

## Quantitative Analysis

### Recall impact of zero-padding at 1536

The SRHT of a d-dim unit vector padded to D dims, subsampled back to d dims, is equivalent to a Johnson-Lindenstrauss projection from D to d. The JL lemma guarantees that for n points and distortion ε:

```
d ≥ C · ln(n) / ε²
```

For 1536 subsampled from 2048, the effective distortion is bounded but nonzero. More practically:

- **25% energy loss** means the MSE reconstruction captures ~75% of the original inner product signal
- The QJL correction partially compensates, but its 1-bit resolution limits recovery
- Empirical expectation: recall@10 drops 1–3% at 1536 vs 2048 for equivalent bit-width, based on the energy retention ratio

For 1024 and 2048 (both power-of-2), zero distortion from padding. The recall difference between them is purely from dimensionality — 2048 encodes more information from the source embedding.

### Storage budget for 1M vectors at 4-bit

| Dimension | Payload/vector | 1M vectors | Index overhead (est. M=8) | Total |
|---|---|---|---|---|
| 1024 | 516 B | 492 MB | ~50 MB | ~542 MB |
| 1536 | 772 B | 736 MB | ~75 MB | ~811 MB |
| 2048 | 1,028 B | 980 MB | ~100 MB | ~1,080 MB |

Moving from 1536 to 2048 adds ~33% storage. Moving to 1024 saves ~33%. For cost-sensitive deployments, 1024 with a Matryoshka-truncated model provides significant savings while maintaining power-of-2 alignment.

## Consequences

### Positive

- **2048 default eliminates the most common source of quantization waste** — no zero-padding for the recommended configuration
- **All FWHT-aligned dimensions get exact codebook match** — no distribution mismatch for 1024 or 2048
- **Tiled FWHT keeps 1536 viable** — users migrating from pgvector/OpenAI don't need to re-embed
- **Three tiers cover the practical embedding model landscape** — quality (2048), compact (1024), compatibility (1536)
- **No wire format changes** — tiling strategy is deterministic from existing datum fields

### Negative

- **2048@4bit spills L1D on Graviton** — 64 KB LUT fills entire L1D, forcing sq+candidate to L2 (2.5x latency). Mitigated by recommending 3-bit for Graviton deployments.
- **2048 default increases storage 33% over 1536 at same bit-width** — ~270 MB per million vectors at 4-bit. But 2048@3bit has identical payload to 1536@4bit (772 B), neutralizing this for the recommended Graviton configuration.
- **Scoring throughput at 2048 drops ~25% vs 1536** — more dimensions per candidate in the hot loop, partially offset by better L1D behavior at 3-bit
- **Tiled FWHT is new code** — increases implementation surface in the quantizer module
- **Codebook parameterization change breaks 1536-dim encoded data** — acceptable pre-release, would need migration post-release
- **3-bit has fewer centroids (4 vs 8)** — higher per-dimension quantization error. The 33% more dimensions and elimination of padding waste are expected to compensate, but this requires empirical validation.

### Neutral

- Compression ratio is nearly identical across tiers (7.77x–7.88x at 4-bit) — the storage difference is proportional to the raw dimension, not the quantization efficiency
- The architecture is already dimension-agnostic — tiers are documentation and benchmarks, not hard constraints
- Future dimensions (768, 3072, 4096) can be added as tiers without architectural changes; tiled FWHT generalizes to any factorization
- Graviton's 64 KB L1D is consistent across all current generations (2/3/4) — this constraint is stable and not expected to change in the near term

## Open Questions

1. **2048@3bit vs 1536@4bit recall comparison**: Both have identical payload (772 B) and total information bits (6,144). Does the FWHT alignment advantage of 2048@3bit overcome the per-dimension quantization loss from fewer centroids? This is the most critical empirical question for this ADR. Benchmark on MTEB or similar with Matryoshka-truncated embeddings.
2. **Tiled FWHT tile size selection**: Should 1536 use 3×512 or 6×256? Larger tiles preserve more decorrelation but have fewer factorization options. Needs empirical recall comparison.
3. **Codebook per tile vs per dimension**: With tiled FWHT, should each tile use a codebook parameterized by tile_dim, or should a single codebook be used? Per-tile codebooks add complexity but match the distribution exactly.
4. **Graviton L1D spill measurement**: What is the actual throughput regression of 2048@4bit vs 2048@3bit on Graviton3/4? The 64 KB LUT is borderline — hardware prefetch may mask the L2 penalty for sequential access patterns. Needs criterion benchmarks on r7g instances.
5. **Matryoshka truncation quality**: What is the empirical recall@10 of 2048-truncated text-embedding-3-large vs native 1536 text-embedding-3-small? This determines whether the dimension upgrade is free or requires re-embedding.
6. **Tiled LUT accumulation**: For configurations where the LUT exceeds L1D (2048@4bit, any@5bit+), would processing the scoring loop in cache-line-sized tiles (e.g., 1024 dims at a time, two passes over the candidate) recover L1D hit rates? Cost is reading the candidate payload twice (~1 KB), which is trivial if it stays in L1.

## References

### Theory & Algorithms
- [TurboQuant: Online Vector Quantization with Near-optimal Distortion Rate (ICLR 2026)](https://arxiv.org/abs/2504.19874) — SRHT rotation and Lloyd-Max quantization theory
- [Matryoshka Representation Learning (NeurIPS 2022)](https://arxiv.org/abs/2205.13147) — truncatable embeddings enabling flexible dimension choice
- [Johnson-Lindenstrauss lemma](https://en.wikipedia.org/wiki/Johnson%E2%80%93Lindenstrauss_lemma) — distortion bounds for random projection / subsampling
- [Weaviate: 8-bit Rotational Quantization](https://weaviate.io/blog/8-bit-rotational-quantization) — tiled (block-diagonal) FWHT in production

### Hardware
- [Arm Neoverse V2 in AWS Graviton 4 — Chips and Cheese](https://chipsandcheese.com/p/arms-neoverse-v2-in-awss-graviton-4) — 64 KB L1D, 2 MB L2, 10-cycle L2 load-to-use latency
- [Hot Chips 2023: Arm Neoverse V2 — Chips and Cheese](https://chipsandcheese.com/p/hot-chips-2023-arms-neoverse-v2) — cache hierarchy and SVE2 details
- [Neoverse V1 microarchitecture — WikiChip](https://en.wikichip.org/wiki/arm_holdings/microarchitectures/neoverse_v1) — Graviton3 core specs
- [AWS Graviton — Wikipedia](https://en.wikipedia.org/wiki/AWS_Graviton) — generation comparison

### Internal
- ADR-006: Own quantizer — SRHT implementation in tqvector
- ADR-020: TurboQuant competitive positioning — compression and recall comparisons
- FR-013: Quantization pipeline — two-stage MSE + QJL specification
