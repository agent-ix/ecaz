---
id: ADR-031
title: "RaBitQ Binary Pre-Filter for Beam Search Candidate Scoring"
status: SUPERSEDED
impact: Affects NFR-001, FR-014, ADR-029, ADR-030
date: 2026-04-12
---
# ADR-031: RaBitQ Binary Pre-Filter for Beam Search Candidate Scoring

## 2026-05-01 Status Update: Superseded By Landed RaBitQ Quantizer

This ADR's narrow "binary prefilter for beam search" framing is superseded.
The useful RaBitQ work landed as a first-class quantizer instead:

- `src/quant/rabitq.rs` owns the reusable quantizer, estimator, error-bound,
  and trait implementation.
- `ec_ivf` supports RaBitQ through `storage_format = 'rabitq'` and
  `quantizer = 'rabitq'`.
- `ecaz quant feasibility` provides the offline recall/error-bound study
  surface.
- Local benchmark docs include IVF RaBitQ rows.

The older HNSW prefilter design remains historical context only. Future RaBitQ
work should build on the landed quantizer/profile surface, not revive this ADR
as a standalone HNSW prefilter plan.

## Context

### The scoring cost dominates warm latency

tqvector's current scorer takes ~14,000ns per candidate. At 300-400 candidates per query,
scoring accounts for ~4-5ms of the ~11ms warm surface. The NFR-001 target is p50 < 5ms.

ADR-029 established that an approximate pre-filter is viable (ρ > 0.999 rank correlation),
but the scalar int8 implementation's 1.7x speedup was insufficient for runtime integration
(packet 275). ADR-030 proposes FastScan grouped scoring at ~60-100ns per score. This ADR
proposes an even faster first-stage filter using binary quantization.

### RaBitQ: binary quantization with theoretical guarantees

RaBitQ (SIGMOD 2024, Apache 2.0 license) quantizes D-dimensional vectors to D-bit binary
codes and scores using XOR + POPCNT operations. Key properties:

1. **Proven optimal space-accuracy tradeoff.** RaBitQ achieves the asymptotically optimal
   bound for binary quantization — no other 1-bit scheme can do better at the same bit
   budget.

2. **POPCNT-based scoring.** Distance estimation uses bitwise XOR of two binary codes
   followed by population count. At 1536 dimensions: 24 × 64-bit XOR + POPCNT operations
   ≈ **~8ns per score**. This is ~1,750x faster than the current exact scorer and ~8-12x
   faster than FastScan PQ4.

3. **Analytical error bound.** RaBitQ provides a theoretical error bound on the distance
   estimate that can be used to calibrate rejection thresholds.

4. **Already proven in production.** Integrated into Elasticsearch/Lucene and Qdrant with
   HNSW, demonstrating compatibility with graph-based ANN search.

### Deriving binary codes from existing 4-bit data

tqvector already stores 4-bit centroid indices per dimension. Each index selects one of 16
codebook values. A binary code can be derived by thresholding the codebook value:

- If `codebook[index] >= 0`: bit = 1
- If `codebook[index] < 0`: bit = 0

This produces a 1536-bit (192-byte) binary code **without storing any additional data** — the
binary code is a function of the existing 4-bit code and the known codebook. At query time,
the query's binary code is derived from the sign of each rotated query dimension.

The XOR + POPCNT of these sign-based codes approximates the inner product: dimensions where
query and candidate agree in sign contribute positively; disagreements contribute negatively.
The correlation with the exact inner product depends on the magnitude distribution, but for
post-FWHT near-Gaussian distributions, sign agreement is a strong signal.

Alternatively, the full RaBitQ algorithm can be applied to generate optimized binary codes
that minimize the theoretical error bound, stored as an additional 192 bytes per element.

## Hypothesis

A RaBitQ-style binary pre-filter using sign-derived or optimized binary codes can:

1. Score all ~300-400 beam candidates in **~2.4-3.2μs total** (vs ~4.2ms current)
2. Reject 70-85% of candidates, keeping ~50-80 survivors for exact rescoring
3. Reduce total scoring cost to ~0.7-1.1ms (binary filter + exact rescore of survivors)
4. Maintain recall@10 within 1pp of the unfiltered baseline

### Zero-storage variant (sign-derived codes)

Derive binary codes from existing 4-bit codes at score time:
- For each packed byte, extract both nibbles, look up codebook sign → 2 bits
- 768 packed bytes → 1536 bits = 192 bytes of binary code
- This extraction adds ~50-100ns per candidate but avoids storing extra data
- The binary codes can be cached per element in the graph element cache

### Stored variant (optimized RaBitQ codes)

Store a 192-byte optimized binary code alongside the existing 768-byte 4-bit code:
- Total element payload: 768 + 192 = 960 bytes (still 6.4x smaller than pgvector's 6,144)
- The binary code is precomputed during index build using the full RaBitQ algorithm
- Eliminates the per-score extraction overhead
- Provides tighter error bounds than the sign-derived variant

## What Not To Assume

1. **Do not assume sign-derived codes are as good as full RaBitQ.** The sign-based binary
   code discards magnitude information entirely. Full RaBitQ uses a carefully calibrated
   normalization and randomized rounding that preserves more distance information in the
   binary representation. The sign-derived variant is a cheap first experiment; full RaBitQ
   may be needed for adequate recall.

2. **Do not assume 1-bit is enough for final ranking.** Binary quantization is a pre-filter,
   not a replacement for exact scoring. The final top-k ranking must use the exact f32 scorer
   (or the FastScan grouped scorer from ADR-030 if that is adopted).

3. **Do not assume POPCNT is always the fastest path.** On ARM (Graviton), the equivalent
   is `vcnt` (NEON byte-level popcount). The instruction throughput may differ from x86
   POPCNT. Benchmark on target hardware.

4. **Do not assume the binary pre-filter integrates at the same point as ADR-029.** Packet
   275 showed that per-source-expansion filtering (16 candidates) is too small a pool for
   the two-stage overhead. A binary pre-filter at ~8ns per score has much lower fixed
   overhead, so per-source filtering may now be viable. But batch filtering across multiple
   expansions may still be better. Test both.

5. **Do not assume the stored variant is necessary.** If the sign-derived codes achieve
   adequate rank correlation (ρ ≥ 0.85) and the extraction cost (~100ns) is amortized by
   caching, the zero-storage variant may be sufficient. Only add the stored codes if the
   sign-derived path fails the recall gate.

## Required Validation

1. **Sign-derived rank correlation.** On the 10K benchmark corpus, derive binary codes from
   the existing 4-bit codes (codebook sign thresholding) and compute Spearman ρ between
   Hamming-distance-based scores and exact f32 scores. Gate: ρ ≥ 0.85 (lower bar than
   ADR-029's 0.9 because this is a coarser first-stage filter intended to be followed by
   exact rescoring with a generous survivor budget).

2. **Full RaBitQ rank correlation.** If sign-derived codes fail the gate, implement the full
   RaBitQ normalization and rounding, store 192-byte codes, and remeasure. Gate: ρ ≥ 0.85.

3. **Survivor capture.** At various survivor budgets (top-20, top-50, top-80 of ~300
   candidates), measure what fraction of the exact top-10 is captured. Gate:
   exact_top10_captured_by_binary_top50 ≥ 0.99 across all test queries.

4. **Per-score microbenchmark.** Measure: (a) sign-derived binary code extraction time,
   (b) POPCNT scoring time, (c) total pre-filter time for 300 candidates. The total must
   be under 10μs to justify the approach.

5. **End-to-end latency.** Measure warm p50 with the binary pre-filter integrated into beam
   search (binary filter → exact rescore). Gate: ≥2ms improvement on the 10K benchmark.

## Decision

**Superseded.** The sign-derived HNSW prefilter investigation was overtaken by
the first-class RaBitQ quantizer and IVF integration. Treat this ADR as
background for the scoring-kernel lineage, not an active decision.

Packet 280 changed the sequencing around ADR-030: grouped FastScan remains interesting, but
only as a larger index-v2 effort with a new grouped encoding. RaBitQ has since
landed through the quantizer/profile seam rather than through the prefilter-only
runtime path described here.

## Consequences

### If confirmed (sign-derived variant)

- No on-disk format change — binary codes derived from existing 4-bit data
- Graph element cache gains a cached binary code field (192 bytes per element)
- A new `score_binary_prefilter` function computes Hamming distance via XOR + POPCNT
- The beam expansion loop gains a two-stage structure: binary filter → exact rescore
- Per-candidate pre-filter cost: ~8ns (POPCNT) + ~100ns (code extraction, amortized by cache)
- Total scoring cost drops from ~4.2ms to ~0.7-1.1ms

### If confirmed (stored variant)

- Element tuple grows by 192 bytes (768 → 960 bytes, still 6.4x smaller than pgvector)
- No per-score extraction overhead — binary code read directly from page
- Tighter error bounds from full RaBitQ algorithm
- Build pipeline gains a binary code computation step

### If rejected

- ADR-030 (FastScan grouped scoring) becomes the primary scoring speedup path
- The binary pre-filter idea is recorded as a failed experiment
- ADR-029's SIMD int8 scorer remains the fallback approximate scoring approach

### Composition with ADR-030

If both ADR-030 and ADR-031 succeed, a three-stage pipeline is possible:

1. **RaBitQ binary filter** (~8ns/candidate): score all ~300 candidates → keep top ~50
2. **FastScan grouped score** (~100ns/candidate): score 50 survivors → keep top ~15
3. **Exact f32 scorer** (~14μs/candidate): score 15 survivors → final top-k

Total: ~2.4μs + ~5μs + ~210μs ≈ **~0.2ms for scoring** — a 20x reduction from the
current ~4.2ms scoring cost.

Until ADR-031 has been run down, treat this composition as follow-on work rather than the next
implementation slice. ADR-030 now assumes a larger encoding/layout redesign after packet 280.

## Selection Criteria (shared with ADR-030)

ADR-030 and ADR-031 are both being investigated. The following criteria determine how
results are used, duplicated from ADR-030 for self-containedness:

### Theoretical basis for each approach

**FastScan (ADR-030)** exploits the structure of product quantization: by grouping dimensions
into subvectors, each with a small codebook, the per-candidate scoring work reduces from
O(D) to O(M) lookups where M = D/group_size. The precomputed per-group LUT moves the
per-dimension multiply into query-prep time (paid once) rather than per-candidate time
(paid 300x). This is an algebraic reordering — the same computation, amortized differently.
The theoretical speedup is D/M = group_size (e.g., 16x for S=16). The uint8 LUT
quantization adds approximation error bounded by the quantization step size relative to
the per-group distance range.

**RaBitQ (ADR-031)** exploits high-dimensional geometry: in high dimensions, normalized
random vectors concentrate on a thin annulus near the unit sphere. Binary quantization
(sign of each coordinate after random rotation) preserves angular relationships because
the expected Hamming distance between two binary codes is a monotone function of the
angle between the original vectors. RaBitQ's theoretical contribution is proving that
this relationship achieves the information-theoretic optimal rate — no 1-bit-per-dimension
scheme can produce a better distance estimator. The speedup comes from replacing
multiply-accumulate with XOR + POPCNT, which processes 64 dimensions per clock cycle.

**Why one might beat the other in practice:**
- FastScan preserves more information (4 bits vs 1 bit per group) so it has inherently
  higher recall quality per score. If the uint8 LUT quantization is tight, FastScan can
  serve as a standalone scorer.
- RaBitQ is faster per-score (POPCNT vs vpshufb) but carries less information, so it
  needs a follow-up rescoring stage. In a pipeline, the total cost depends on the survivor
  ratio — if RaBitQ can reject 80%+ of candidates cheaply, the pipeline wins even though
  the surviving candidates need expensive exact rescoring.
- At small expansion pools (2m=16 candidates per source), the two-stage overhead may
  dominate the per-score savings. FastScan's single-stage architecture avoids this problem.
  RaBitQ's speed advantage only materializes when the candidate pool is large enough to
  amortize the stage-transition cost.

### Head-to-head selection (if only one is adopted)

| Criterion | FastScan (ADR-030) | RaBitQ (ADR-031) | Winner |
|---|---|---|---|
| Per-score speed | ~100ns | ~8ns | RaBitQ |
| Recall quality per score | Higher (4-bit grouped) | Lower (1-bit binary) | FastScan |
| Can replace exact scorer for beam search | Yes, if ρ ≥ 0.995 | No — always needs exact rescore | FastScan |
| Implementation complexity | Higher (vpshufb + LUT build + optional batching) | Lower (XOR + POPCNT) | RaBitQ |
| On-disk format change | None | None (sign-derived) or +192 bytes (stored) | Tie / FastScan |
| Pipeline simplicity | Single-stage possible | Always two-stage | FastScan |

### Composition decision

If both succeed, compose as a three-stage pipeline only if the measured end-to-end latency
of the composed pipeline beats the best single-approach latency. Do not add pipeline stages
for theoretical elegance — measure the overhead of each stage transition and confirm it
doesn't eat the scoring savings (as happened in packet 275).

## References

- [RaBitQ Paper (SIGMOD 2024)](https://arxiv.org/abs/2405.12497) — Apache 2.0
- [RaBitQ Library](https://github.com/VectorDB-NTU/RaBitQ-Library) — Apache 2.0
- [Extended RaBitQ (SIGMOD 2025)](https://github.com/VectorDB-NTU/Extended-RaBitQ) — Apache 2.0
- [Qdrant Binary Quantization](https://qdrant.tech/articles/binary-quantization/) — production
  deployment reference
- [Elastic BBQ](https://www.elastic.co/search-labs/blog/better-binary-quantization-lucene-elasticsearch) —
  RaBitQ in Elasticsearch/Lucene
- ADR-029: Compressed-domain approximate scoring — int8 variant, ρ > 0.999 validated
- ADR-030: FastScan grouped subvector scoring — complementary mid-tier scorer
- Packet 274: ADR-029 rank correlation study — validates approximate scoring approach
- Packet 275: Source-expansion survivor gate — showed scalar 1.7x insufficient; binary
  pre-filter at ~1,750x may overcome this
