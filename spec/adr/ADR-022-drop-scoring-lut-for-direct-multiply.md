---
id: ADR-022
title: "Drop Scoring LUT in Favor of Direct Codebook Multiply"
status: PROPOSED
impact: Affects FR-014, NFR-001, B1 SIMD task
date: 2026-04-08
---
# ADR-022: Drop Scoring LUT in Favor of Direct Codebook Multiply

## Context

The current `score_ip_from_split_parts` hot path in `src/quant/prod.rs` uses a precomputed lookup
table (LUT) to score each candidate. At query preparation time, `prepare_ip_query` builds a table
of `dim * num_centroids` floats containing `codebook[centroid] * rotated_query[dim]` for every
(dimension, centroid) pair. At scoring time, each dimension does a single indexed LUT read instead
of a multiply.

This is the standard Asymmetric Distance Computation (ADC) pattern from the product quantization
literature, where codebooks typically have 64-256 centroids and the LUT fits comfortably in L1.

TurboQuant's architecture is different. Quality comes from three stages — SRHT rotation, scalar
Lloyd-Max quantization, and gamma-weighted QJL residual correction — not from a large codebook.
At the current `bits = 4` configuration, there are only `2^(bits-1) = 8` centroids. This changes
the cost tradeoff:

| | LUT approach (current) | Direct multiply |
|---|---|---|
| Codebook footprint | N/A (baked into LUT) | 8 floats = **32 bytes** (one cache line) |
| Per-query allocation | `dim * 8 * 4` bytes = **48 KB** at 1536-dim, **64 KB** at 2048-dim | None |
| L1 pressure | LUT pushes or exceeds typical 32-64 KB L1d | Codebook permanently resident |
| Per-dimension scoring op | One indexed LUT read (strided by 8) | One codebook read + one FMA |
| SIMD vectorization | Requires `_mm256_i32gather_ps` (6-12 cycles on most x86) or scalar gather | Load 8 query values contiguously, gather 8 codebook entries (tiny table), `_mm256_fmadd_ps` |
| Query preparation cost | `O(dim * centroids)` multiplies + allocation | Zero (codebook and query stored directly) |

The direct multiply approach trades one FMA per dimension (1-cycle throughput on modern x86) for
eliminating 48-64 KB of per-query allocation and replacing strided LUT gathers with sequential
query reads plus tiny-table codebook lookups.

## Hypothesis

For TurboQuant's 8-centroid configuration, dropping the LUT and scoring via direct
`codebook[index] * rotated_query[dim]` will:

1. Reduce per-query memory from 48-64 KB to near zero
2. Improve L1 cache residency during multi-candidate scoring
3. Enable simpler and faster SIMD vectorization of the scoring loop
4. Produce bit-identical quality (same computation, different order)

## What Not To Assume

1. **Do not assume the LUT is always wrong.** If future work increases centroid count (e.g., mixed
   bit-widths, higher-quality tiers), the LUT tradeoff may shift back. The decision should be
   re-evaluated if `num_centroids` exceeds ~32.

2. **Do not assume the multiply is free.** At very high candidate throughput, the extra FMA may
   matter. Benchmarking must compare total scoring throughput, not just single-candidate latency.

3. **Do not optimize the MSE loop alone.** The QJL sign-accumulation loop
   (`score_ip_from_split_parts` lines 189-197) should be fused into the same pass and vectorized
   together. The sign-bit check can become a branchless SIMD blend.

## Required Validation

1. **Microbenchmark:** `score_ip_encoded` throughput (candidates/sec) at 1536-dim and 2048-dim,
   LUT vs direct multiply, both scalar and AVX2.
2. **Cache analysis:** Compare L1d miss rates under sustained scoring (e.g., `perf stat` or
   equivalent) for LUT vs direct paths.
3. **SIMD prototype:** Demonstrate that the direct-multiply inner loop vectorizes cleanly with
   `_mm256_fmadd_ps` and does not require gather instructions.
4. **Bit-exactness:** Confirm that LUT and direct-multiply produce identical scores on the existing
   test corpus.

## Decision

**Open.** Investigate as part of B1 SIMD scoring work. Do not remove the LUT until benchmarks
confirm the direct-multiply path is faster at the target operating points.

## Consequences

### If confirmed
- `PreparedQuery` drops the `lut` field; scoring reads `codebook` + `rotated_query` directly
- Per-query allocation shrinks by 48-64 KB
- SIMD scoring becomes a straightforward FMA loop instead of a gather-based LUT scan
- Query preparation gets faster (no LUT precomputation)

### If rejected
- Keep the LUT but document that it is justified by measured throughput, not by convention
- Consider a hybrid: LUT for sequential scan, direct multiply for graph traversal (where L1
  pressure from graph page reads competes with the LUT)
