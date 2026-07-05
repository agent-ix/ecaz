# Task 148: TQ estimator bias corrections (length renormalization + codebook calibration)

Status: **proposed** (2026-07-04). Owner: unassigned.
Priority: P2 — the recall levers for the two TQ niches that survived the
Task 124/147 pareto sweeps.

## Why

The Task 124 packet 001 / Task 147 evidence leaves TQ with exactly two
product niches, and both are limited by raw estimator quality — the one
regime where rerank masking does NOT save a coarse estimator:

1. **TQ no-rerank default** (zero heap fetches): trails rb1+heap_f32 by
   ~4.6 pp recall at 1m n32 (0.9208 vs 0.9667).
2. **TQ stage-2 payload** (`coarse_rerank` + index/turboquant +
   stage2@25): gives up 0.3–1.0 pp recall at 1m because the TQ stage
   mis-ranks within the width-50 frontier, so the top-25 exact set
   occasionally drops truth rows.

Two standard corrections for rotate-then-Lloyd-Max quantizers are
missing from our estimator, both cheap:

- **Length renormalization (RaBitQ-derived).** Lloyd-Max conditional-
  mean decoding systematically shortens vectors (law of total variance:
  `||x̂||² ≈ ||x||² − γ²`), and the shrinkage varies PER VECTOR with its
  quantization error, so it distorts cross-candidate ranking. The fix is
  one per-vector scalar multiplied back at scoring time. **We already
  compute the ingredient at encode time**: `gamma` = residual L2 norm
  (`src/quant/prod.rs` encode; surfaced at
  `src/am/ec_ivf/quantizer.rs:217`) — and the production no-QJL scoring
  paths explicitly discard it (`let _ = gamma`). Correction factor
  ≈ `||x|| / ||x̂||` (derivable from gamma + decoded-code norm, or
  persisted directly).
- **Per-coordinate calibration pre-pass.** Our Lloyd-Max codebook is
  computed from the THEORETICAL post-SRHT Beta marginal
  (`src/quant/codebook.rs`), parameterized only by dimension. Real
  embeddings deviate, and ADR-024 already documents a known mismatch:
  the tiled-512 FWHT at 1536-dim shifts the marginal (Beta(256,256) vs
  Beta(768,768)). A build-time calibration pass fits the codebook (or a
  per-coordinate scale) to sampled data; our `training_sample_rows`
  plumbing could host it.

Context worth keeping: asymmetric scoring we already have — LUT scoring keeps the query in full precision; the int8_approx
default's query-side quantization costs ≤0.42 pp (Task 144 A/B). And
the corrections do NOT reopen sub-4-bit TQ: TQ2/binary failed with a
dedicated correction scorer (`qjl2_32`, pruned by Task 130); at 1 bit
the corrected construction IS RaBitQ (rb1).

## Scope

Measurement-first, in slices with per-change A/B attribution:

1. **Bias audit (cheap, no code change to product paths).** Instrument
   or offline-compute the estimator bias on staged corpora: distribution
   of `<q,x̂>/<q,x>` and of `||x̂||/||x||` per vector at 4-bit no-QJL.
   If the per-vector spread is negligible, close the renorm branch as a
   source-grounded negative without touching the scorer.
2. **Length renormalization A/B.** Apply the per-vector scale on the
   no-QJL 4-bit scoring paths (LUT + int8_approx: fold the scalar into
   the per-candidate epilogue; the SDOT accumulator is unchanged).
   Decide persistence: recompute-from-gamma vs a persisted f32/f16
   scalar (dense-block layout change needs an ADR per the format
   invariants). A/B at 10k/50k/100k(+1m): recall + latency + storage,
   (a) TQ no-rerank default, (b) stage2@25 cell vs the Task 124 packet
   001 baseline.
3. **Codebook calibration A/B.** Build-time per-coordinate fit (scale
   or codebook refit) from `training_sample_rows` samples; same A/B
   matrix, measured separately from slice 2 (no stacking).
4. Gate: corrections must be ~latency-neutral on the int8/SDOT path
   (the Task 143 defaults' win must not regress).

## Out of Scope (hard)

- No sub-4-bit TQ formats (settled: Task 130 prune, Task 147 rb2 cell).
- No default flips in this task — promotion evidence feeds the standing
  rb1-vs-TQ default-format question instead.
- QJL re-enablement (separate, already-measured surface).

## Gate / Exit Criteria

- Slice-1 bias audit numbers in the packet (or the negative).
- If pursued: per-slice A/B at 10k/50k/100k minimum on both target
  niches, with the stage2@25 1m recall gap re-measured, and a clear
  keep/drop verdict per correction.

## References

- Task 124 packet: `reviews/task-124/001-stage2-vs-rb1-pareto/`
- Task 147 packet: `reviews/task-147/001-density-pareto/`
- ADR-006 (quantizer origin), ADR-007 (gamma contract), ADR-018 (QJL),
  ADR-024 (tiled-FWHT marginal mismatch), ADR-025 (bit allocation,
  "why 3+1 underperforms")
- Task 130 (sub-4-bit prune record)
