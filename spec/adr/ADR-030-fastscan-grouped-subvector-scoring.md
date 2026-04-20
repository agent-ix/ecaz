---
id: ADR-030
title: "FastScan Grouped Subvector Scoring for 4-bit Codes"
status: PROPOSED
impact: Affects NFR-001, FR-014, ADR-029, ADR-024
date: 2026-04-12
---
# ADR-030: FastScan Grouped Subvector Scoring for 4-bit Codes

> **2026-04-12 study update:** Packet 280 did not validate the "reinterpret the current
> scalar 4-bit format as grouped FastScan" path on the real corpus. ADR-030 remains worth
> exploring, but no longer as a drop-in scorer on the current encoding. From this point
> forward, treat ADR-030 as an index-v2 direction built around a new grouped search-code
> layout, and defer that work until ADR-031 has been run down.
>
> **2026-04-13 v2 checkpoint:** ADR-030 now has a concrete versioned-v2 direction:
> transformed grouped `PQ4` search codes, a hot binary sidecar, an optional cold rerank payload,
> and a query pipeline of `binary prefilter -> grouped FastScan -> tiny rerank`. Do not spend more
> time on current-format grouped reinterpretation unless a new, specific reason emerges.

## 2026-04-13 Design Checkpoint

ADR-030 is no longer "a faster scorer for today's code bytes." It is a new index format.

### Transform front-end

The v2 metadata should support both:

- `SRHT`
- `OPQ`

The first implementation path should still start with `SRHT` because tqvector already has that
transform, and the first question is whether true grouped `PQ4` is strong enough on transformed
data to justify the redesign at all. `OPQ` is the leading follow-on quality lever if `SRHT`
grouped `PQ4` is promising but not strong enough.

### Grouped code structure

For the current `1536`-dim lane, the default target grouped code is:

- `96` subvectors
- `16` dims per subvector
- `4` bits per subvector
- one learned 16-centroid codebook per subvector

That produces a `48B` grouped search code per vector and matches the FastScan/QuickerADC-style
scoring model much better than tqvector's current scalar code stream.

### Persisted payloads

ADR-030 should assume distinct persisted payloads for distinct runtime jobs:

1. **hot grouped search code** for FastScan-style traversal
2. **hot binary sidecar** for cheap candidate rejection
3. **cold higher-fidelity rerank payload** for a very small survivor set

The pragmatic first rerank payload is the existing scalar `4-bit` tqvector code kept as a cold
sidecar. That reuses a scorer we already have while letting the hot search path stay compact. A
later v2 follow-up can replace the cold payload with a better residual / `PQ8` contract if data
shows it is worth the extra complexity.

### Page layout

The hot scan tuple should keep only data that the search loop reads often:

- graph-local tuple state
- hot binary sidecar
- hot grouped search code

The cold rerank payload should live separately so layer-0 scans do not read a larger rerank blob
for every candidate. So ADR-030 requires a new hot/cold tuple/page locality plan, not merely a new
SIMD kernel.

### Query-time scoring pipeline

The intended pipeline is:

1. `ADR-031`-style binary prefilter on the hot binary sidecar
2. grouped FastScan scorer on the hot grouped `PQ4` payload
3. tiny rerank on the cold higher-fidelity payload

If later measurements show the grouped scorer is strong enough to stand alone for traversal, the
binary stage can become optional. But the design should initially assume the composed pipeline,
because it currently has the best odds of clearing the target frontier.

### Versioning / migration

Treat v2 as rebuild-only. The metadata page needs explicit format/version information instead of
assuming that all indexes share today's tuple contract.

At minimum, v2 metadata should carry:

- `format_version`
- transform kind and transform parameters
- grouped-code configuration
- payload-presence flags for binary/search/rerank payloads

Do not auto-upgrade v1 indexes in place.

### First feasibility spike

The first bounded experiment should stay offline and answer the highest-risk question:

> does true grouped `PQ4` on transformed tqvector data have enough ranking quality to justify the
> redesign?

Concretely:

1. extend `src/bin/approx_score_study.rs`
2. train true grouped codebooks on transformed vectors
3. start with `SRHT`
4. compare `f32` vs quantized LUT scoring to isolate LUT loss from encoding loss
5. compare against fp32 truth on the same overlap/capture metrics already used in packet `280`

Only after that feasibility spike is positive should ADR-030 move into persisted layout and runtime
integration slices.

## 2026-04-14 Sequencing Update From Review Feedback

Reviewer feedback on packets `310-333` does not materially change the ADR-030 v2 design. It does
clarify what has to be interleaved before grouped-v2 can move from "experimental build lane" to a
real query path.

### What stays the same

The intended steady-state architecture remains:

1. transformed grouped `PQ4` search code
2. hot binary sidecar
3. cold higher-fidelity rerank payload
4. query pipeline of `binary prefilter -> grouped FastScan -> tiny rerank`

### What changes in sequencing

Do not treat the remaining work as "finish scorer, then clean up safety items later."

Interleave the following before grouped-v2 leaves the experimental gate:

1. **Shared grouped encoder contract**
   - remove or prove equivalent the duplicate grouped-code packing paths in `build.rs` and
     `approx_score_study.rs`
   - make grouped training determinism explicit enough for regression checks
2. **Insert/vacuum format safety**
   - add explicit grouped-v2 rejection or grouped-aware handling in `src/am/insert.rs`
   - add explicit grouped-v2 rejection or grouped-aware handling in `src/am/vacuum.rs`
3. **Cold rerank fetch**
   - add a real `reranktid -> cold tuple` read seam before claiming the hot/cold split is complete
4. **Stronger metadata/runtime validation**
   - validate required grouped-v2 metadata fields at scan-open
   - preserve the current scan-side rejection point until the grouped scorer is intentionally
     enabled
5. **End-to-end quality measurement**
   - re-measure the full `binary -> grouped -> rerank` path on real data before any gate-lift
     decision

### Explicit gate-lift blockers

Grouped-v2 must not leave the experimental build gate until all of the following exist:

- grouped scorer on real scan inputs
- cold rerank fetch
- end-to-end recall/latency measurement on the full pipeline
- insert-path grouped-v2 safety
- vacuum-path grouped-v2 safety
- shared grouped encoder contract or equivalent cross-path proof

### Advisory follow-ups

These are lower urgency than the blockers above, but are still valid review outcomes:

- clarify metadata naming around `bits` vs rerank bit-width semantics
- document that `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` is a build-time gate, not a kill switch
- keep raw-page validation always-on for v2 builds and fail builds loudly on mismatch
- remove helper-path allocations from grouped hot-path tuple accessors before grouped scoring is
  enabled

## Context

### The per-dimension scoring bottleneck

tqvector's current scorer (`score_ip_from_split_parts_no_qjl_4bit`, `src/quant/prod.rs:238`)
iterates over all 1536 dimensions individually: extract nibble → codebook load → multiply →
accumulate. This takes ~14,000ns per score due to the dependent codebook load creating a serial
dependency chain (~4 cycle L1 latency per dimension).

pgvector's f32 dot product scores the same 1536 dimensions in ~60-100ns by auto-vectorizing a
contiguous float array with AVX2 FMA. The 140x per-score gap is the primary reason tqvector at
10K warm (~11ms) is slower than pgvector at 1M at 90% recall (~9.5ms).

ADR-029 proposed an int8 approximate scorer as a pre-filter. Packet 274 validated the rank
correlation (ρ > 0.999), but the scalar int8 implementation achieved only 1.7x speedup — not
enough for the per-source integration point tested in packet 275.

### How FAISS FastScan solves this

FAISS FastScan (Meta, MIT license) reorganizes 4-bit product quantization codes so scoring
operates on **grouped subvectors** rather than individual dimensions:

1. **Group dimensions into subvectors.** Instead of 1536 independent 4-bit codes, treat
   the data as M subvectors of S dimensions each (e.g., M=96 subvectors of S=16 dims).
   Each subvector is assigned one of 16 centroids (4-bit code), just like today.

2. **Precompute a per-subvector distance table at query time.** For each subvector m and
   each centroid c, compute `lut[m][c] = Σ_{d in subvec_m} codebook[m][c][d] * query[d]`.
   This table has M × 16 entries. At M=96: 96 × 16 = 1,536 entries × 1 byte (quantized
   to uint8) = 1.5KB — fits in L1.

3. **Score using vpshufb lookups.** The 16-entry LUT for one subvector fits in one 128-bit
   register. `vpshufb` uses the 4-bit code as a selector to look up the precomputed
   distance, processing 32 vectors × 2 subvectors per SIMD instruction pair.

4. **Batch candidates for SIMD throughput.** Codes are laid out in blocks of 32 vectors,
   interleaved by subvector, so the SIMD pipeline processes 32 candidates simultaneously.

Scoring cost: M = 96 LUT lookups + 96 additions ≈ **~60-100ns per score**, matching
pgvector's throughput while reading 768 bytes instead of 6,144.

### Compatibility with TurboQuant

tqvector's existing 4-bit codes are per-dimension scalar quantization: each dimension is
independently quantized to one of 16 centroids from a global codebook. The SRHT rotation
decorrelates dimensions before quantization, making them approximately independent.

FastScan assumes **per-subvector** centroids: each group of S dimensions shares a local
codebook trained on that subvector's distribution. This is a different quantization scheme.

However, tqvector's scalar quantization can be **reinterpreted** as a degenerate grouped PQ
where each subvector of S dimensions uses the same global codebook applied independently per
dimension. The per-subvector LUT can be built by summing the per-dimension contributions
within each group. This reinterpretation does not change the on-disk format or the encoded
values — only the scoring algorithm changes.

The quality difference: standard grouped PQ with learned per-subvector codebooks can capture
within-subvector correlations that scalar quantization misses. But after SRHT rotation,
within-subvector correlations should be minimal (that's the purpose of the rotation), so the
quality gap may be small.

### Post-study correction

Packet 280 weakens the compatibility claim above for tqvector's current encoding. On the real
`ec_hnsw_real_10k` corpus, grouped reinterpretation of the existing per-dimension scalar codes
lost too much ranking quality at the same time that the expected scorer speedup was not strong
enough to justify the approximation. The practical conclusion is:

1. ADR-030 is still compatible with the broader TurboQuant philosophy of "rotate first, search
   in the compressed domain."
2. ADR-030 is not compelling as a reinterpretation of today's scalar 4-bit code stream.
3. A serious ADR-030 implementation should assume a new grouped encoding and storage layout,
   not just a new scorer over the current tuples.

## Hypothesis

Reinterpreting tqvector's existing 1536 × 4-bit codes as 96 groups of 16 dimensions and
scoring with precomputed per-group LUT lookups via `vpshufb` can:

1. Reduce per-score cost from ~14,000ns to ~60-200ns (70-230x speedup)
2. Maintain bit-identical scores when the LUT is computed in f32 (the reinterpretation
   changes the computation order, not the result)
3. Enable competitive warm latency at 10K by making scoring cost comparable to pgvector's
   f32 dot product

This hypothesis did not hold for the current scalar-code format in packet 280. Keep the
performance target, but do not assume it is reachable without a new grouped encoding.

## What Not To Assume

1. **Do not assume the reinterpretation is lossless.** The per-group LUT quantizes the
   precomputed distances to uint8 for SIMD accumulation. This introduces quantization error
   in the distance estimate. The error is small when per-group distance values have a narrow
   range, but must be measured.

2. **Do not assume vpshufb gives the full theoretical speedup.** The 128-bit vpshufb operates
   per-lane in 256-bit AVX2 mode. The code layout for batch processing (32 vectors interleaved
   by subvector) requires careful data reorganization. The implementation complexity may
   limit the realized speedup.

3. **Do not assume this replaces the exact scorer.** If the uint8 LUT quantization introduces
   rank-order errors, this scorer serves as a fast pre-filter (like ADR-029) rather than a
   replacement. The exact f32 scorer would still rescore the final candidates.

4. **Do not assume M=96 is optimal.** Smaller subvectors (M=192, S=8) give a larger LUT
   but finer-grained grouping. Larger subvectors (M=48, S=32) give a smaller LUT but
   coarser grouping with more per-group quantization error. The optimal M should be
   determined empirically.

5. **Do not assume batch-of-32 layout is required.** FAISS batches 32 vectors for SIMD
   throughput, which requires reorganizing the code storage layout. tqvector could start
   with a per-candidate vpshufb scorer (no batching) and add batching later if needed.

## Required Validation

1. **LUT construction correctness.** On the 10K benchmark corpus, verify that scoring via
   the per-group f32 LUT produces bit-identical results to the current per-dimension scorer
   (same computation, different grouping order). This validates the reinterpretation.

2. **uint8 LUT rank correlation.** Quantize the per-group LUT to uint8 and measure Spearman
   rank correlation against exact f32 scores. Gate: ρ ≥ 0.99 (same bar as ADR-029 study).

3. **Per-score microbenchmark.** Measure per-score latency for: (a) current f32 nibble scorer,
   (b) per-group f32 LUT scorer (no SIMD), (c) vpshufb uint8 LUT scorer. The vpshufb path
   must achieve ≥20x speedup over the current scorer to justify the implementation complexity.

4. **End-to-end latency.** Measure warm p50 with the grouped scorer integrated into beam
   search. Gate: ≥2ms improvement on the 10K benchmark.

5. **Subvector size sweep.** Test S ∈ {8, 16, 32} to find the Pareto-optimal point on the
   accuracy-vs-speed curve.

## Decision

**Deferred after ADR-031.** Packet 280 was enough to reject the "current encoding, new scorer"
version of ADR-030 as the immediate C1 path. We still want to explore ADR-030, but only as a
larger index-v2 effort after ADR-031 has been fully run down.

That deferred ADR-030 should assume work across multiple subsystems:

1. a new grouped search-code encoding, likely after SRHT or a learned rotation
2. new tuple/page layout for grouped codes and any sidecar search metadata
3. build-path changes to train or derive grouped codebooks and emit the new payloads
4. new runtime scoring kernels and likely a versioned compatibility story

## Consequences

### If confirmed

- `PreparedQuery` gains a grouped LUT field (96 × 16 uint8 entries = 1.5KB)
- A new `score_grouped_fastscan_no_qjl_4bit` function scores using vpshufb lookups on grouped
  search codes
- Per-score cost drops from ~14μs to ~0.1-0.2μs — comparable to pgvector's f32 dot product
- The beam search expansion loop can score all candidates cheaply without a two-stage filter,
  potentially simplifying the ADR-029 pipeline
- ADR-024's move to 2048 dims adds ~33% more subvectors (128 instead of 96) but the per-score
  cost remains ~0.1-0.2μs
- The warm 10K surface could drop from ~11ms to ~7-8ms (scoring shrinks from ~4ms to ~0.03ms,
  with allocation/graph overhead becoming the dominant cost)

### If rejected

- The per-dimension scoring architecture remains
- ADR-029's int8 approximate scorer (with future SIMD) becomes the primary speedup path
- ADR-031 (RaBitQ binary pre-filter) provides an alternative fast pre-filter

### While deferred

- ADR-031 is the active near-term research and runtime path
- ADR-030 remains a tracked follow-on, but not a drop-in optimization on the current index
- Any resumed ADR-030 work should start from the "new grouped encoding" premise rather than
  retrying the current scalar-code reinterpretation

## Selection Criteria (shared with ADR-031)

ADR-030 and ADR-031 are both being investigated. Both may succeed independently, or they
may compose into a multi-stage pipeline. The following criteria determine how results are
used:

### Single-stage vs multi-stage decision

| Criterion | Threshold | Implication |
|---|---|---|
| FastScan rank correlation (ρ vs exact f32) | ≥ 0.995 | FastScan can serve as the **sole beam scorer** — no pre-filter or exact rescore needed during beam search. Exact rescore only at final top-k emission. |
| FastScan rank correlation (ρ vs exact f32) | 0.95–0.995 | FastScan is a **mid-tier scorer** — adequate for beam traversal but exact rescore needed for final top-k. RaBitQ pre-filter adds value only if candidate pool is large. |
| FastScan rank correlation (ρ vs exact f32) | < 0.95 | FastScan is a **pre-filter only** — similar role to RaBitQ but slower. Prefer RaBitQ as pre-filter, use exact scorer for beam decisions. |

### Head-to-head selection (if only one is adopted)

| Criterion | FastScan (ADR-030) | RaBitQ (ADR-031) | Winner |
|---|---|---|---|
| Per-score speed | ~100ns | ~8ns | RaBitQ |
| Recall quality per score | Higher (4-bit grouped) | Lower (1-bit binary) | FastScan |
| Can replace exact scorer for beam search | Yes, if ρ ≥ 0.995 | No — always needs exact rescore | FastScan |
| Implementation complexity | Higher (vpshufb + LUT build + optional batching) | Lower (XOR + POPCNT) | RaBitQ |
| On-disk format change | None | None (sign-derived) or +192 bytes (stored) | Tie / FastScan |
| Pipeline simplicity | Single-stage possible | Always two-stage | FastScan |
| Risk if recall insufficient | Falls back to pre-filter role | Falls back to nothing — pre-filter is its only role | FastScan (more graceful degradation) |

### Composition decision

If both succeed, compose as a three-stage pipeline only if the measured end-to-end latency
of the composed pipeline beats the best single-approach latency. Do not add pipeline stages
for theoretical elegance — measure the overhead of each stage transition (sort/select
survivors) and confirm it doesn't eat the scoring savings (as happened in packet 275).

## References

- [FAISS FastScan Wiki](https://github.com/facebookresearch/faiss/wiki/Fast-accumulation-of-PQ-and-AQ-codes-(FastScan)) — MIT license
- [FAISS Implementation Notes](https://github.com/facebookresearch/faiss/wiki/Implementation-notes) — vpshufb details
- [QuickerADC](https://github.com/technicolor-research/faiss-quickeradc) — vpshufb-based PQ in FAISS
- ADR-022: Drop Scoring LUT — related LUT analysis (the 96KB per-dimension LUT was too big;
  the per-group LUT at 1.5KB is 64x smaller)
- ADR-024: FWHT Transform Strategy — dimension count affects group count
- ADR-029: Compressed-domain approximate scoring — complementary approach
- Packet 265: Removed 96KB LUT — the per-group LUT avoids this problem by being 64x smaller
- Packet 274: ADR-029 rank correlation study — validates int8 quantization for this codebook
- Packet 280: ADR-030 grouped feasibility study — current scalar-code reinterpretation is not
  strong enough to justify immediate adoption
