---
id: ADR-030
title: "FastScan Grouped Subvector Scoring for 4-bit Codes"
status: PROPOSED
impact: Affects NFR-001, FR-014, ADR-029, ADR-024
date: 2026-04-12
---
# ADR-030: FastScan Grouped Subvector Scoring for 4-bit Codes

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

## Hypothesis

Reinterpreting tqvector's existing 1536 × 4-bit codes as 96 groups of 16 dimensions and
scoring with precomputed per-group LUT lookups via `vpshufb` can:

1. Reduce per-score cost from ~14,000ns to ~60-200ns (70-230x speedup)
2. Maintain bit-identical scores when the LUT is computed in f32 (the reinterpretation
   changes the computation order, not the result)
3. Require no changes to the on-disk format or encoding pipeline
4. Enable competitive warm latency at 10K by making scoring cost comparable to pgvector's
   f32 dot product

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

**Open.** The LUT construction correctness check (validation step 1) is the prerequisite.
If the f32 reinterpretation is not bit-identical (due to floating-point reordering), measure
the error magnitude and determine whether it's acceptable.

## Consequences

### If confirmed

- `PreparedQuery` gains a grouped LUT field (96 × 16 uint8 entries = 1.5KB)
- A new `score_grouped_fastscan_no_qjl_4bit` function scores using vpshufb lookups on the
  existing packed 4-bit codes without changing the on-disk format
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
