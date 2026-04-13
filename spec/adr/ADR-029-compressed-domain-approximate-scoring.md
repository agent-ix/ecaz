---
id: ADR-029
title: "Compressed-Domain Approximate Scoring for Beam Search Pre-Filter"
status: PROPOSED
impact: Affects NFR-001, FR-014, ADR-028, ADR-022
date: 2026-04-12
---
# ADR-029: Compressed-Domain Approximate Scoring for Beam Search Pre-Filter

## Context

### The per-score cost gap with pgvector

pgvector's HNSW scorer is a contiguous f32 dot product that auto-vectorizes with AVX2 FMA:

```c
// pgvector vector.c:601 — VECTOR_TARGET_CLONES generates FMA clone
for (int i = 0; i < dim; i++)
    distance += ax[i] * bx[i];
```

At 1536 dimensions with `vfmadd231ps` (8 floats/cycle), this runs in **~60–100ns per score**.
pgvector also scores zero-copy from the pinned buffer page — no allocations.

tqvector's scorer (`score_ip_from_split_parts_no_qjl_4bit`, `src/quant/prod.rs:238`) iterates
over 768 packed bytes, extracting each nibble, doing a dependent codebook load, and multiplying:

```rust
let low_nibble = (packed & 0x0F) as usize;
sum += self.codebook[low_nibble] * prepared.rotated[dim_index];
```

The codebook load creates a **serial dependency chain** (~4 cycle L1 latency per dimension).
For 1536 dimensions this takes **~14,000ns per score** — roughly 140x slower than pgvector's
dot product. Combined with 3–4 heap allocations per element load (`.to_vec()`, `Vec<ItemPointer>`,
`Vec<u8>`, `Arc::new()`), the total per-candidate cost is ~16,000ns vs pgvector's ~500–1,000ns.

At 300–400 candidates per query, scoring alone accounts for ~4–5ms of the 11ms warm surface.

### The fundamental problem

The current scorer **decompresses per dimension**: extract nibble → load f32 from codebook →
multiply by f32 query value → accumulate. This is equivalent to decompressing the entire vector
one element at a time and computing a full-precision dot product, except with worse memory
access patterns than a contiguous f32 array.

The 4-bit quantized format stores 1536 dimensions in 768 bytes using only 16 possible centroid
values per dimension. This is a tiny alphabet. The scorer should exploit that smallness rather
than treating each dimension as an independent codebook lookup.

### Relationship to ADR-028

ADR-028 proposes scoring fewer dimensions (partial pre-filter) using the same per-dimension
scorer. This ADR proposes scoring all dimensions but with a radically cheaper scorer that
operates directly on the compressed representation. The two are complementary — a cheap
approximate scorer could be combined with a dimension budget — but this ADR addresses the
more fundamental problem: the per-dimension cost, not the dimension count.

## Design Space

### Approach 1: SIMD nibble lookup with `vpshufb`

With 16 centroids, each fitting in a nibble, `_mm_shuffle_epi8` / `_mm256_shuffle_epi8`
(`vpshufb`) can look up 16/32 table entries simultaneously using nibble indices as selectors.

**Concept:**
1. At query prep, quantize the 16 `codebook[c] * scale` products to int8 values (one-time)
2. Load the 16 int8 values into a 128-bit register (the lookup table)
3. Load 16 packed bytes = 32 nibbles = 32 dimensions of candidate data
4. Use `vpshufb` to look up all 32 int8 approximate values at once
5. Similarly prepare an int8 version of the rotated query
6. Multiply int8 × int8 using `_mm256_maddubs_epi16` → int16 partial sums
7. Accumulate in int32

**Throughput estimate:** Processing 32 dimensions per `vpshufb` + `maddubs` takes ~3–4 cycles.
For 1536 dims: ~48 groups × ~4 cycles = **~192 cycles ≈ ~60–100ns**. This matches pgvector's
throughput while operating on 768 bytes instead of 6,144.

**Accuracy:** Quantizing the codebook×query products to int8 introduces quantization error.
With 16 centroid values and a well-behaved post-rotation distribution, the relative error is
small. The int8 approximate score should have high rank correlation with the exact f32 score.

### Approach 2: Tiled micro-LUT

The removed 96KB LUT (`packet 265`) precomputed `lut[d][c] = codebook[c] * rotated[d]` for
all 1536 × 16 entries. This turned scoring into pure table lookups (no multiplies) but the
96KB LUT polluted L2 cache.

**Concept:** Process dimensions in tiles of T (e.g., T=64). For each tile:
1. Build a micro-LUT of T × 16 f32 entries = T × 64 bytes
2. Score the tile's packed bytes using the micro-LUT (pure lookups, no multiplies)
3. Advance to the next tile

At T=64, each micro-LUT is 4KB — fits comfortably in L1. Total tiles: 1536/64 = 24.
Each tile rebuild is 64 × 16 = 1024 multiplies. Total rebuilds: 24 × 1024 = 24,576 multiplies
(same as the old big LUT build). But the scoring loop within each tile is pure lookups with
sequential access — no dependent codebook loads, no multiplies.

**Trade-off:** The micro-LUT amortizes the build cost across many candidates if the tile LUT
is reused. But in beam search, each candidate has different packed data — the LUT must be
rebuilt per tile per query (not per candidate). So the build cost is paid once per query, and
each candidate's scoring is pure lookups. This may be faster than the current approach if the
lookup loop auto-vectorizes better than the nibble-extract-multiply loop.

### Approach 3: Bit-parallel approximate score

With 4-bit codes, represent the query in quantized form and compute approximate inner products
using integer arithmetic on the packed byte stream directly.

**Concept:**
1. At query prep, for each centroid c, compute `weight[c] = codebook[c] * mean(|rotated|)`
2. Quantize weights to int4 or int8
3. For each packed byte in the candidate, both nibbles select weights from the quantized table
4. Accumulate using SIMD integer additions

This is the crudest approximation but processes the packed data with minimal decode overhead.

## Hypothesis

Approach 1 (vpshufb int8 approximate scoring) can serve as a **fast pre-filter** in the beam
search expansion loop:

1. Compute an int8 approximate score for each candidate in ~60–100ns (vs ~14,000ns for exact)
2. Reject 60–80% of candidates whose approximate score falls below the beam worst threshold
3. Run the exact f32 scorer only on surviving candidates (~20–40% of total)
4. Net scoring cost drops from ~4–5ms to ~1–2ms per query
5. Recall loss ≤ 1pp (bounded by the int8 quantization error and rejection threshold)

The approximate score should correlate well with the exact score because:
- The codebook has only 16 values — int8 quantization of 16 distinct values is nearly lossless
- The rotated query values follow a near-Gaussian post-FWHT distribution that int8 captures well
- The accumulation in int32 avoids overflow for 1536 dimensions

## What Not To Assume

1. **Do not assume int8 quantization is lossless.** The codebook values and rotated query values
   have different dynamic ranges. The int8 mapping must be calibrated to preserve rank ordering,
   not absolute values. Measure Spearman rank correlation between int8-approximate and f32-exact
   scores on the benchmark corpus.

2. **Do not assume vpshufb is available everywhere.** SSSE3 provides `_mm_shuffle_epi8` (128-bit),
   AVX2 provides `_mm256_shuffle_epi8` (256-bit, but per-lane). ARM NEON provides `vtbl`/`vqtbl`.
   A scalar fallback is needed and should still be faster than the current scorer because it
   replaces f32 codebook loads + f32 multiplies with int8 table lookups + int8 multiplies.

3. **Do not assume the approximate scorer replaces the exact scorer.** The approximate score is
   a filter, not a substitute. Final beam results must use exact f32 scores for correct ranking
   and recall. The architecture is: approximate score → threshold → exact score for survivors.

4. **Do not assume approach 1 is better than approach 2.** The tiled micro-LUT (approach 2)
   avoids the int8 quantization error entirely and may auto-vectorize well with modern compilers.
   Both should be prototyped.

## Required Validation

1. **Rank correlation study.** On the 10K benchmark corpus, compute both exact f32 and int8-
   approximate scores for all candidate pairs encountered during beam search. Report Spearman
   ρ. Gate: ρ ≥ 0.9 (higher bar than ADR-028's 0.7 because this approximation replaces the
   full scorer for filtering, not just a dimension subset).

2. **Microbenchmark.** Measure per-score latency for: (a) current f32 nibble scorer, (b) vpshufb
   int8 approximate scorer, (c) tiled micro-LUT scorer. Report cycles and ns at 1536 dims.

3. **End-to-end latency.** Measure warm p50 with the two-stage scoring pipeline (approximate →
   threshold → exact) vs current single-stage. Gate: ≥2ms improvement on the 10K benchmark.

4. **Recall impact.** Run the recall harness at ef_search=40 with the two-stage pipeline at
   multiple rejection thresholds. Gate: ≤1pp recall@10 loss.

## Decision

**Open.** The rank correlation study and microbenchmark are prerequisites. If int8 approximate
scores do not correlate well with exact scores (ρ < 0.9), investigate the tiled micro-LUT
(approach 2) before abandoning the direction.

## Consequences

### If confirmed

- `PreparedQuery` gains int8-quantized fields for the approximate scorer
- A new `approximate_score_no_qjl_4bit` function scores using vpshufb or integer arithmetic
  directly on `mse_packed` bytes without decompressing to f32
- The beam expansion loop in `load_layer0_successor_candidates` gains a two-stage structure:
  fast approximate score → threshold check → exact f32 score for survivors
- Per-candidate cost drops from ~14,000ns to ~100ns for rejected candidates and ~14,100ns for
  accepted candidates, yielding an overall reduction proportional to the rejection rate
- Warm surface drops by an estimated 2–4ms, representing the largest single improvement path
  remaining for NFR-001
- Combined with allocation reduction (zero-copy decode), the warm surface could approach
  5–7ms — within striking distance of the NFR-001 p50 < 5ms target

### If rejected

- The scorer remains single-stage f32
- ADR-028's partial-dimension approach becomes the primary path for reducing scoring cost
- The 140x per-score gap with pgvector persists, limiting tqvector's competitiveness at
  small scale where compression does not provide a cache advantage

## References

- ADR-022: Drop Scoring LUT — related scorer pipeline change
- ADR-028: Partial-dimension pre-filter — complementary approach (fewer dims, same scorer)
- Packet 265: Removed 96KB LUT that was polluting L2 — informs approach 2 (tiled micro-LUT)
- Packet 266: AVX2 scorer failure — vpshufb approach is structurally different (int8 lookup
  vs f32 permute/blend) and avoids the per-lane 16-entry problem that killed the AVX2 path
- pgvector source: `src/vector.c:601` VectorInnerProduct, `src/hnswutils.c:534`
  HnswLoadElementImpl — reference implementation for zero-copy scoring from pinned pages
