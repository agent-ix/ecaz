# Task 136: TQ rank-1 in-register scorer for IVF (int8_approx wiring)

Status: **proposed** (2026-07-02). Owner: unassigned. Priority: P2
Follow-up to Tasks 125/132/133; highest-ceiling scorer idea on the table.

## Why

The no-QJL 4-bit LUT is a **rank-1 outer product**: `lut[d][c] = codebook[c] *
rotated_query[d]` (`src/quant/prod.rs` `build_prepared_query_lut*`, analytic
16-entry codebook shared by all dims). The 48 KiB i16 LUT that Task 125 proved
is L1D-critical materializes 24,576 products of two vectors totalling ~1.5 KB.
A factored kernel keeps the codebook in ONE register (`vqtbl1q`) and streams an
i8 rotated query — it already exists as `int8_approx32`
(`src/quant/int8_approx32/{neon,avx2,sve}.rs`, Task 98) and is wired into HNSW
behind `ec_hnsw.turboquant_exact_score_mode=int8_approx`, but **not into IVF**,
whose prepared-query path hardcodes the LUT (`src/am/ec_ivf/quantizer.rs`).
Rough NEON instruction count is ~3× lower than the LUT block kernel and the
query-side working set drops ~30×. Task 133 measured the scorer at 46% of the
IVF approximate scan — the single largest stage.

## Scope

- Wire an int8_approx prepared-query variant into the IVF scorer dispatch
  (query-side only; the on-disk 4-bit codes are decoded exactly as today).
- A/B vs the LUT path at 10k/50k/100k (`ecaz bench suite`, recall + latency +
  storage, stage + kernel counters). Recall guard: Task 98 measured a
  0.2–0.4 pp dip for int8_approx on HNSW (i8-quantized rotated query); if the
  dip exceeds noise at 50k/100k, evaluate the recall-safe i16 factored variant
  (dequantize codebook to i16 in-register via `vqtbl2q`, i16 rotated query,
  `vmull_s16` — error bounded by the already-shipped i16 LUT rounding) before
  concluding.

## Out of Scope (hard)

- No new on-disk format/mode/reloption; query-side representation only.
- No default flip without the full 10k/50k/100k recall+latency evidence.

## Gate / Exit Criteria

- A measurable IVF latency win at recall within noise of the LUT path
  (ships as default or as documented opt-in), or a source-grounded negative
  recording where the factored kernel loses (recall or latency).
