---
id: ADR-025
title: "Bit Allocation Within the MSE+QJL Quantizer: 3+1 vs 4+0 vs 4+1"
status: PROPOSED
impact: Affects FR-013, NFR-002, NFR-003, ADR-007, ADR-021, ADR-024
date: 2026-04-09
---
# ADR-025: Bit Allocation Within the MSE+QJL Quantizer — 3+1 vs 4+0 vs 4+1

## Context

ADR-006 already chose tqvector's quantizer family: a TurboQuant-style two-stage
MSE+QJL quantizer rather than the earlier alternative. ADR-007 then fixed the
payload and scoring contract around that architecture: persisted packed MSE
indices, persisted QJL bits, and persisted `gamma` for the residual correction.

This ADR does **not** reopen that architectural decision. The open question here
is narrower: given that tqvector already uses an MSE+QJL quantizer family, how
should the available bits be allocated between the MSE stage and the QJL stage
for the supported operating points?

That question is explicitly in scope for ADR-007's follow-up benchmarking
program. ADR-007 requires MSE-only ablations, payload/latency/index-size
measurement, and allows follow-on ADRs when benchmarks show the v0.1 allocation
or storage assumptions should be revisited. ADR-021 and ADR-024 both consume the
current allocation assumptions, but neither is the right home for the actual bit
allocation decision:

- ADR-021 is about recommended dimension tiers and operating points.
- ADR-024 is about transform strategy and codebook alignment for non-power-of-2
  dimensions.
- ADR-025 is therefore the follow-on ADR that settles bit allocation **within**
  the existing MSE+QJL architecture.

The recall profiling campaign (reviews 200–204) tested three allocation
strategies with significantly different recall characteristics:

| Scheme | MSE bits | QJL bits | Total bits/dim | Centroids |
|---|---|---|---|---|
| **3+1** | 3 | 1 | 4 | 8 |
| **4+0** | 4 | 0 | 4 | 16 |
| **4+1** | 4 | 1 | 5 | 16 |

The 3+1 baseline (current `bits - 1` allocation in `prod.rs:52`) did not recall well. The 4+0
configuration (tiled FWHT path where QJL is disabled, `prod.rs:297-299`) showed substantially
better recall. Other TurboQuant-family implementations (TurboQuantDB, Weaviate) use 4+1.

The original storage target was **4 bits per dimension** to achieve ~7.8x compression versus
fp32. Moving to 4+1 adds a fifth bit. This ADR evaluates the downstream implications of that
fifth bit on payload size, L1/L2 cache optimization, page density, and scoring throughput.

### Why 3+1 Underperforms

With 3-bit MSE (8 centroids), each dimension is quantized to one of 8 levels. The Lloyd-Max
codebook for the post-rotation Beta distribution places these centroids to minimize MSE, but
8 levels provides coarse coverage of the distribution. The 1-bit QJL stage can only correct
the sign of the residual — it cannot recover magnitude information lost to coarse quantization.

With 4-bit MSE (16 centroids), the per-dimension quantization error drops significantly.
Doubling the centroid count provides finer coverage of the distribution tails, where
informative dimensions concentrate after rotation.

### Why Other Implementations Use 4+1

TurboQuantDB and Weaviate both allocate 4 bits to MSE and 1 bit to QJL independently,
totaling 5 bits per dimension. The rationale:

1. 4-bit MSE provides adequate centroid resolution (16 levels)
2. QJL provides an unbiased correction term for the residual (ADR-018)
3. The QJL bit is cheap (1/5 of total storage) relative to its recall contribution
4. The 5-bit total still achieves ~6.3x compression versus fp32

## Analysis

### Payload Size Comparison

At 1536 dimensions:

| Scheme | MSE bytes | QJL bytes | gamma | **Payload** | Compression |
|---|---|---|---|---|---|
| 3+1 | 576 B | 192 B | 4 B | **772 B** | 7.85x |
| 4+0 | 768 B | 0 B | 4 B | **772 B** | 7.85x |
| **4+1** | **768 B** | **192 B** | **4 B** | **964 B** | **6.28x** |

MSE at 4-bit: `ceil(1536 × 4 / 8) = 768 B`. QJL: `ceil(1536 / 8) = 192 B`. The fifth bit
adds exactly one QJL bit-plane — **+192 bytes per vector (+25%)**.

Across all dimension tiers:

| Dim | 4-byte payload | 5-byte payload | Δ | 5-byte compression |
|---|---|---|---|---|
| 1024 | 516 B | 644 B | +128 B (+25%) | 6.25x |
| 1536 | 772 B | 964 B | +192 B (+25%) | 6.28x |
| 2048 | 1,028 B | 1,284 B | +256 B (+25%) | 6.29x |

The storage cost is proportional: +25% uniformly across all dimensions.

### L1 Cache Impact — The Critical Constraint

The LUT is the dominant consumer of L1 cache during scoring. It scales as
`dim × centroids × sizeof(f32)`. Doubling centroids from 8 to 16 **doubles the LUT**.

**LUT sizes:**

| Scheme | Centroids | LUT @ 1024 | LUT @ 1536 | LUT @ 2048 |
|---|---|---|---|---|
| 3+1 (3-bit MSE) | 8 | 32 KB | 48 KB | 64 KB |
| 4+0 / 4+1 (4-bit MSE) | 16 | 64 KB | 96 KB | 128 KB |

**Scoring hot-path working set on Graviton (64 KB L1D):**

| Component | 1024 (3+1) | 1024 (4+1) | 1536 (3+1) | 1536 (4+1) | 2048 (3+1) | 2048 (4+1) |
|---|---|---|---|---|---|---|
| LUT | 32 KB | **64 KB** | 48 KB | **96 KB** | 64 KB | **128 KB** |
| sq vector | 4 KB | 4 KB | 6 KB | 6 KB | 8 KB | 8 KB |
| Candidate | 516 B | 644 B | 772 B | 964 B | 1 KB | 1.3 KB |
| **Total** | **36.5 KB** | **68.6 KB** | **54.8 KB** | **103 KB** | **73 KB** | **137 KB** |
| **L1D util** | 57% | **107%** | 86% | **161%** | 114% | **214%** |

**At 4+1, every dimension tier spills L1D on Graviton.** Even 1024-dim — previously the most
comfortable tier at 57% utilization — now exceeds 100%. The LUT alone fills the entire 64 KB
L1D at 1024 dimensions.

This is a qualitative change, not merely quantitative. ADR-021 established that L1D fit is the
binding constraint for scoring throughput on Graviton, with L2 load-to-use latency at ~10
cycles (2.5x penalty versus the ~4-cycle L1 hit). The LUT access pattern is data-dependent
(centroid index varies per dimension), so hardware prefetching cannot fully mask the penalty.

**Estimated scoring throughput regression:**

| Dim | 3+1 throughput | 4+1 throughput | Regression |
|---|---|---|---|
| 1024 | ~143K/s | ~85–95K/s | -33% to -40% |
| 1536 | ~95K/s | ~50–60K/s | -37% to -47% |
| 2048 | ~71K/s | ~35–40K/s | -44% to -50% |

For comparison, ADR-021 Table "Working set sizes at 5-bit (16 centroids)" already flagged this
exact scenario: "At 5+ bits, even 1024 starts to pressure L1D." The 4+1 scheme is functionally
equivalent to 5-bit from the LUT's perspective — 4 MSE bits produce 16 centroids regardless of
whether QJL is present.

### Page Density Impact

Element tuple size at 4+1 (adding 192 B to the 1536-dim element):

| Dim | 4-byte element | 4+1 element | Tuples/page | Δ tuples | 1M index size | Δ size |
|---|---|---|---|---|---|---|
| 1024 | 586 B | 714 B | 11 (was 13) | -15% | ~660 MB | +22% |
| 1536 | 842 B | 1,034 B | 7 (was 9) | -22% | ~1,012 MB | +25% |
| 2048 | 1,098 B | 1,354 B | 5 (was 7) | -29% | ~1,350 MB | +25% |

Fewer tuples per page means more page reads during HNSW traversal. At 1536@4+1, the 1M-vector
index crosses 1 GB, increasing shared_buffers pressure on RDS instances (ADR-021 §Page I/O).

### The 2048@3+1 Equivalence Is Broken

ADR-021 identified that 2048@3bit and 1536@4bit share identical payload (772 B) and total
information bits (6,144). This equivalence was a key argument for recommending 2048@3bit as
the Graviton-optimal configuration.

At 4+1, the comparison becomes:

| Configuration | Payload | LUT | L1D util | Energy retained | Codebook match |
|---|---|---|---|---|---|
| 1536 @ 4+1 | 964 B | 96 KB | 161% | 100% (tiled) | No |
| 2048 @ 3+1 | 772 B | 32 KB | 64% | 100% | Yes |

2048@3+1 is now **smaller** (20% less payload), **faster** (fits L1D comfortably), and
**better aligned** (power-of-2, no codebook mismatch). The case for 1536@4+1 over 2048@3+1
weakens substantially — 1536@4+1 would need to demonstrate a meaningful recall advantage over
2048@3+1 to justify its costs.

### Comparison with ADR-021 5-Bit Analysis

ADR-021 §Working set sizes at 5-bit already evaluated the 16-centroid LUT scenario:

> | Component | 1024 | 1536 | 2048 |
> |---|---|---|---|
> | LUT | 64 KB | 96 KB | 128 KB |
> | **L1D utilization** | **100% — borderline** | 150% — spills | 200% — badly spills |
>
> At 5+ bits, even 1024 starts to pressure L1D.

The 4+1 scheme produces the same LUT pressure as 5-bit because the LUT is determined by MSE
centroids alone. QJL does not use the LUT — it uses the `sq` vector for scoring. However, 4+1
adds the `sq` vector and QJL candidate bytes *on top of* the 5-bit-class LUT, making L1D
pressure slightly worse than pure 5-bit MSE.

## Mitigation Strategies

If 4+1 is required for recall quality, several strategies can partially recover the cache
penalty:

### 1. Tiled LUT scoring (ADR-021, Open Question #6)

Process the scoring loop in L1-sized chunks: score 512 dimensions at a time, keeping each LUT
tile (512 × 16 × 4 = 32 KB) in L1D. The candidate payload is read once per tile (~1 KB per
pass). For 1536 dimensions, this means 3 passes over the candidate.

**Expected recovery**: Significant. Each 32 KB LUT tile fits in L1D alongside the sq vector
slice and candidate slice. The cost is re-reading the candidate payload per tile (~3 KB total
extra reads for 1536-dim), which is trivial relative to the L2 penalty it avoids.

**Complexity**: Moderate. Requires restructuring the inner scoring loop from a single linear
pass to a tiled iteration. The QJL accumulation loop is already independent and can remain
untiled.

### 2. Fix codebook parameterization first (zero cost, ADR-024 §Decision 1)

The codebook mismatch bug (Beta(768) vs Beta(256) for tiled FWHT) can yield +2–5pp recall
at zero storage cost. This should be measured before committing to the fifth bit, as it may
close enough of the recall gap to make 4+0 viable.

### 3. Hybrid: 4+0 for graph traversal, 4+1 for rerank

Store 4+1 payloads on disk but use only the 4-bit MSE codes during HNSW graph traversal
(where throughput and L1D fit matter most). Apply the QJL correction only in the final rerank
pass over the top-k candidates (typically 10–100 vectors).

**Advantage**: Graph traversal stays at 4+0 cache profile. The rerank pass touches few enough
candidates that L2 latency is acceptable.

**Disadvantage**: QJL correction does not improve graph-edge quality. If QJL primarily helps
distinguish close candidates (rather than gross ranking), this may be sufficient. If QJL
is needed for correct graph navigation, this loses most of the benefit.

### 4. Accept 2048@3+1 as the production configuration

This sidesteps the entire 4+1 question for the recommended tier:

- Same 772 B payload as the original 1536@4-byte target
- 32 KB LUT — 50% L1D utilization on Graviton
- Full FWHT alignment, 100% energy retention, exact codebook match
- Requires Matryoshka-truncated embeddings for models that don't natively produce 2048-dim

1536@4+1 would then exist only as a compatibility tier for users who cannot re-embed.

## Decision

**DEFERRED** pending empirical measurements. The following experiments are required before
committing to a bit allocation strategy:

### Required Measurements

1. **Codebook fix recall delta** (ADR-024 §Decision 1): Rerun the exact-only 1k harness with
   corrected codebook parameterization (`tile_dim` instead of `dim`). This isolates the
   free-recall gain available before any bit-budget increase.

2. **4+0 vs 4+1 recall comparison**: On the same harness, compare 4-bit MSE without QJL
   versus 4-bit MSE with 1-bit QJL. This measures the marginal recall value of the fifth bit.

3. **2048@3+1 vs 1536@4+1 recall comparison**: The critical head-to-head. If 2048@3+1
   matches or exceeds 1536@4+1, the fifth bit is unnecessary for the recommended tier.

4. **Tiled LUT scoring prototype**: If 4+1 is adopted, implement tiled LUT scoring and
   measure actual throughput recovery on Graviton to validate the mitigation strategy.

5. **Structured data validation**: All comparisons should be run on real embeddings (OpenAI
   text-embedding-3-small at 1536, text-embedding-3-large truncated to 2048) in addition to
   synthetic corpora. Review 201 identified that uniform-random data is adversarial for
   quantized recall — relative rankings may change on structured data.

### Decision Framework

- If the codebook fix + 4+0 closes to within **2pp** of 4+1 on real embeddings:
  **stay at 4 bits**. The cache and storage savings dominate.

- If 4+1 is required and 2048@3+1 matches 1536@4+1:
  **recommend 2048@3+1** as the production configuration. 1536@4+1 becomes a compatibility
  tier only.

- If 4+1 is required and 1536@4+1 meaningfully exceeds 2048@3+1:
  **adopt 4+1 with tiled LUT scoring**. Accept the 25% storage increase and validate that
  tiled LUT scoring recovers L1D throughput.

## Consequences

### If 4+1 is adopted

**Storage**: +25% payload per vector. Compression drops from ~7.8x to ~6.3x versus fp32.
Still a substantial improvement over uncompressed storage, but a regression from the original
4-byte target.

**Cache**: L1D spills at all dimension tiers on Graviton. Tiled LUT scoring becomes mandatory
for latency-sensitive workloads. Without it, scoring throughput regresses 33–50%.

**Page density**: 15–29% fewer tuples per 8 KB page depending on dimension. More page reads
during HNSW traversal. Larger indexes increase shared_buffers pressure.

**Architecture**: Tiled LUT scoring adds complexity to the inner scoring loop. The QJL
accumulation path is unchanged.

### If 4-byte target is maintained

**Recall**: Must be validated as adequate on real embeddings with corrected codebook. The
current synthetic-corpus results are pessimistic (review 201).

**Simplicity**: No LUT tiling needed. Current scoring loop structure is preserved. L1D
behavior matches ADR-021 analysis.

## References

- ADR-007: Persist gamma and raw-query scoring — payload layout and storage targets
- ADR-021: Default vector dimension 2048 — L1D cache analysis, 5-bit LUT evaluation, 2048@3bit equivalence
- ADR-024: FWHT transform strategy — codebook mismatch fix, tiled vs full FWHT
- ADR-018: HNSW graph quality with quantized distances — QJL unbiased estimator property
- ADR-006: Own quantizer — SRHT implementation, extraction from TurboQuantDB
- ADR-020: Embedding dimension operating points — dimension tier analysis
- Review 200–204: A4 recall gate experiments — empirical recall measurements
- TurboQuantDB: upstream 4+1 implementation reference
- Graviton cache specs: 64 KB L1D across all generations (Chips and Cheese, Neoverse V2)
