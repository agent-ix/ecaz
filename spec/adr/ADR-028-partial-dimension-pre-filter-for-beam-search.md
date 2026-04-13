---
id: ADR-028
title: "Partial-Dimension Pre-Filter for Beam Search Candidate Scoring"
status: PROPOSED
impact: Affects NFR-001, FR-014, ADR-022, ADR-024
date: 2026-04-12
---
# ADR-028: Partial-Dimension Pre-Filter for Beam Search Candidate Scoring

## Context

After packets 265–269, the warm steady-state ordered scan for `10K` vectors at `m=8`,
`ef_search=40` sits at `p50≈11.0ms`. The NFR-001 target is `p50 < 5ms`. The remaining ~11ms
is genuine AM execution time — EXPLAIN overhead, statement planning, and beam bookkeeping have
all been ruled out as contributors.

The scoring hot path (`score_ip_from_split_parts_no_qjl_4bit` in `src/quant/prod.rs:238`)
iterates over all 768 packed bytes (1536 dimensions) for every candidate. At ef_search=40, the
beam search evaluates ~300–500 candidates per query, each requiring a full 1536-dimension score.
At ~14μs per score call, scoring accounts for an estimated 4–7ms of the 11ms surface.

Most candidates scored during beam expansion are ultimately rejected — they don't enter the
result set. If a cheap partial-dimension score could reject a large fraction of these candidates
before the full score is computed, the total scoring cost would drop roughly in proportion to
the rejection rate.

### Why this is an architectural decision

A partial-dimension pre-filter introduces a **new stage** in the scoring pipeline: candidates
are first scored on a dimension subset, and only those passing a threshold proceed to the full
scorer. This has three consequences that go beyond an implementation detail:

1. **Recall trade-off.** Any pre-filter that rejects candidates before computing their exact
   score can produce false negatives — a true top-k neighbor may be rejected by the partial
   score. The acceptable recall loss must be bounded.

2. **Pipeline structure change.** The beam search expansion loop in `load_layer0_successor_candidates`
   (`src/am/graph.rs:117`) currently calls `score_candidate` once per element. A pre-filter adds
   a two-stage evaluation: partial score → threshold check → optional full score. This changes the
   `ScoreFn` contract and potentially the `PreparedQuery` struct.

3. **Dimension subset selection.** The choice of which dimensions to evaluate in the pre-filter
   determines the quality/speed tradeoff. This is a design decision with multiple viable
   approaches, each with different recall characteristics.

## Design Space

### Dimension subset strategies

| Strategy | Dimensions evaluated | Basis | Pros | Cons |
|---|---|---|---|---|
| **First-N** | First `k` of 1536 rotated dims | FWHT concentrates energy in early coordinates | Zero cost to select; exploits existing rotation | Energy concentration depends on input distribution; tiled FWHT (3×512) may not concentrate well |
| **Top-variance** | `k` dims with highest codebook variance | Offline selection from training data | Maximizes discriminative power per dimension | Requires precomputed index; changes if codebook changes |
| **Random projection** | `k` random linear combinations | Johnson-Lindenstrauss | Distribution-independent; provable distortion bounds | Requires a projection matrix; adds compute to query prep |
| **Tile-first** | First `k` dims from each 512-dim tile | Matches tiled FWHT structure | Samples all three tiles; simple to implement | Arbitrary choice within each tile |

### Threshold strategies

| Strategy | Description | Pros | Cons |
|---|---|---|---|
| **Absolute** | Reject if partial score < fixed threshold | Simplest | Threshold depends on data distribution |
| **Relative to beam worst** | Reject if partial score < α × beam worst full score, scaled by dim ratio | Adapts to query difficulty | Requires careful scaling factor calibration |
| **Top-fraction pass-through** | Score all candidates partially, keep top `f%` for full scoring | Bounded compute savings | Requires scoring all candidates partially before any full scoring (batching change) |

## Hypothesis

For the no-QJL 4-bit production path (1536 dims, 16 centroids), scoring the first 192 dimensions
(25% of total, 96 packed bytes) as a pre-filter can:

1. Reject 50–70% of beam expansion candidates before full scoring
2. Reduce total scoring time by 30–50% (saving 1.5–3.5ms on the warm surface)
3. Maintain recall@10 within 1 percentage point of the unfiltered baseline

The first-N strategy is the natural starting point because the FWHT rotation (even tiled 3×512)
redistributes energy away from the original embedding's coordinate bias. The first 192 dimensions
of a 512-dim tile contain ~37% of the tile's coordinates, which under the near-uniform
post-rotation distribution should capture a proportional share of the discriminative signal.

## What Not To Assume

1. **Do not assume first-N is optimal.** The tiled FWHT produces three independent 512-dim
   blocks. Energy concentration within each tile depends on the input data. First-N within a
   single tile ignores two-thirds of the rotated space. The tile-first variant (sampling from
   all three tiles) may have better recall at the same dimension budget.

2. **Do not assume the partial score correlates well with the full score.** If the post-rotation
   coordinate variances are highly uniform (which the FWHT aims for), any dimension subset of
   size `k` explains roughly `k/d` of the score variance. A 25% subset would explain ~25% of
   variance — the Pearson correlation between partial and full scores would be ~0.5. This may
   not be discriminative enough to reject candidates confidently. **Measure the rank correlation
   on real data before committing.**

3. **Do not assume the pre-filter is free.** Partial scoring still requires loading the
   candidate's `mse_packed` data from the graph element. If graph loading (not scoring) dominates,
   the pre-filter saves only the compute delta between 96 bytes and 768 bytes of packed data —
   roughly 672 multiplies (~0.3μs at modern throughput). With 300 candidates, this saves ~90μs
   total, which is noise. **The pre-filter only helps if scoring is a larger fraction of per-
   candidate cost than graph loading.**

4. **Do not change the scorer hot path without re-measuring.** The current
   `score_ip_from_split_parts_no_qjl_4bit` is simple and branch-predictor-friendly. Adding a
   conditional early-exit at dimension 192 may disrupt the loop's optimization. A separate
   `partial_score` function called before the full scorer is safer.

## Required Validation

1. **Rank correlation study.** On the 10K benchmark corpus, compute both full and partial (first
   192-dim) scores for all candidate pairs encountered during beam search. Report the Spearman
   rank correlation. If ρ < 0.7, the first-N strategy is unlikely to work and alternative
   strategies should be evaluated before proceeding.

2. **Recall impact.** Run the recall harness at ef_search=40 with the pre-filter at multiple
   rejection thresholds. Report recall@10 vs unfiltered baseline. The acceptance gate is
   ≤1pp recall loss.

3. **Latency impact.** Measure warm p50 with and without the pre-filter on the standard 10K
   benchmark. The pre-filter must show ≥1ms improvement at the recall-acceptable threshold to
   justify the complexity.

4. **Dimension budget sweep.** Test partial dimensions at 96 (12.5%), 192 (25%), 384 (50%)
   to find the Pareto-optimal point on the recall-vs-speedup curve.

## Decision

**Open.** The rank correlation study (validation step 1) is the prerequisite. If partial scores
do not correlate well with full scores under the tiled FWHT rotation, this approach should be
abandoned rather than tuned — no amount of threshold adjustment fixes a weak signal.

## Consequences

### If confirmed (ρ ≥ 0.7, ≤1pp recall loss, ≥1ms improvement)

- `PreparedQuery` gains a `partial_dim: usize` field (or the pre-filter dimension is a
  quantizer-level constant)
- A new `partial_score_no_qjl_4bit` function scores only the first `partial_dim` dimensions
- The beam expansion loop gains a two-stage structure: partial score → threshold → full score
- The `ScoreFn` closure in `load_layer0_successor_candidates` becomes a two-phase callback
  or the pre-filter is applied inline before the closure
- Warm surface drops by 1–3ms, closing a meaningful fraction of the NFR-001 gap

### If rejected (ρ < 0.7 or recall loss > 1pp or improvement < 1ms)

- The scoring pipeline remains single-stage
- Future optimization shifts to reducing the *number* of score calls (tighter beam convergence,
  better entry point selection) rather than making individual calls cheaper
- Alternative pre-filter approaches (random projection, learned hash) could be evaluated but
  carry higher implementation complexity

## References

- ADR-022: Drop Scoring LUT — related scorer pipeline change
- ADR-024: FWHT Transform Strategy — determines the post-rotation energy distribution that
  governs pre-filter effectiveness
- NFR-001: Warm steady-state latency target (p50 < 5ms)
- Packet 264: Warm steady-state optimization survey — idea #7 (partial pre-filter)
- Packet 265: Disable unused query prep — established the 11ms baseline
- Packet 266: AVX2 no-QJL scorer — confirmed scorer is near-optimal, shifting strategy to
  reducing call count
