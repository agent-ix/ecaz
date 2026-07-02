# Task 137: int8_approx SDOT kernel (NEON dotprod upgrade)

Status: **proposed** (2026-07-02). Owner: unassigned. Priority: P2
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
- 1m evidence is deferred to Task 139's promotion matrix (which is gated on
  this task).

## Out of Scope (hard)

- No AVX2-VNNI analog for the Intel lane in this task (record as a
  possible 137b if the NEON win lands).
- No i8mm/SMMLA experiments; SDOT first.
- No on-disk or prepared-query format change.

## Gate / Exit Criteria

- SDOT path bit-equal to the scalar reference in the existing
  `int8_approx32` parity tests (block32, partial, extreme-value, dim-tail).
- Measured scorer_batch reduction at 10k/50k/100k with recall unchanged
  (bit-equal scores ⇒ recall must be byte-identical; any diff is a bug),
  or a source-grounded negative (e.g. kernel already memory-bound).
