# Task 141: int8_approx SDOT kernel (NEON dotprod upgrade)

Status: **review accepted / measured** (2026-07-03; feedback
`reviews/task-141/001-sdot-kernel/feedback/2026-07-03-02-reviewer.md`).
Owner: Codex. Priority: P2

Evidence: `reviews/task-141/001-sdot-kernel/` (code `2d98ec5b7`, branch
`task-141-sdot-kernel`): scorer_batch 15.9→9.0 / 26.6→14.2 / 25.8→13.5
ms/sweep (−44/−47/−47.5%) at 10k/50k/100k, e2e mean 0.76→0.67 / 1.55→1.29 /
2.34→1.95 ms (−11.8/−16.8/−16.7%), recall byte-identical (bit-exact kernel).
Follow-up to Task 136 (int8_approx IVF wiring); serves the Task 98 HNSW
surface too.

## Why

The int8_approx32 NEON kernel (`src/quant/int8_approx32/neon.rs`) predates
the ARM dot-product extension: per 32 dims it issues 4× `vmull_s8` + 4×
`vpadalq_s16` on the de-interleaved even/odd query vectors. `vdotq_s32`
(feature `dotprod`, present on every Apple M-series core and Graviton 2+)
replaces each mull/padal pair with one instruction — ~1.3–1.8× kernel
ceiling. Task 136 measured the current kernel at 25.2 ms/sweep for the
100k IVF scorer stage (already −35% vs the LUT); SDOT is the next scorer
step, worth an estimated further ~5–8% e2e at 100k.

Integer i32 dot accumulation is exact and order-independent, so the
existing strict `.to_bits()` parity tests against the scalar reference
remain the acceptance contract (hamming32-style; no tolerance framing).

## Scope

- Add a `dotprod`-detected fast path in `int8_approx32/neon.rs`
  (`is_aarch64_feature_detected!("dotprod")`), falling back to the current
  NEON path. Keep the analytic codebook `vqtbl1q` dequant and the
  `vld2q_s8` even/odd query layout.
- The kernel serves both consumers unchanged: HNSW
  (`ec_hnsw.turboquant_exact_score_mode=int8_approx`) and IVF
  (`ec_ivf.turboquant_scorer=int8_approx`).
- A/B per the closeout rule: int8_approx pre-SDOT vs post-SDOT, same
  session/tables/binary-pair, IVF 10k/50k/100k recall+latency+storage with
  `ivf_stage_counters`; cite the `scorer_batch` stage delta.
- 1m evidence is deferred to Task 143's promotion matrix (which is gated on
  this task).

## Out of Scope (hard)

- No AVX2-VNNI analog for the Intel lane in this task (record as a
  possible 141b if the NEON win lands).
- No i8mm/SMMLA experiments; SDOT first.
- No on-disk or prepared-query format change.

## Gate / Exit Criteria

- SDOT path bit-equal to the scalar reference in the existing
  `int8_approx32` parity tests (block32, partial, extreme-value, dim-tail).
- Measured scorer_batch reduction at 10k/50k/100k with recall unchanged
  (bit-equal scores ⇒ recall must be byte-identical; any diff is a bug),
  or a source-grounded negative (e.g. kernel already memory-bound).
