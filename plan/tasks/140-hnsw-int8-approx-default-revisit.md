# Task 140: HNSW int8_approx exact-score default revisit

Status: **proposed** (2026-07-02). Owner: unassigned. Priority: P3
Follow-up to Tasks 98 (int8_approx kernel, HNSW wiring) and 136 (IVF
evidence); sweetened by Task 137 (SDOT) if it lands first.

## Why

Task 98 measured a 0.2–0.4 pp recall dip for
`ec_hnsw.turboquant_exact_score_mode=int8_approx` on HNSW and left the
default at `exact`. Task 136 then showed the identical query-side i8
quantization is noise-level on IVF at 10k/50k/100k (max −0.31 pp, inside
ci95) with a −33/−35% scorer-stage win. That weakens the original caution:
the HNSW dip may also be within noise on the current staged dbpedia
corpora, and the designed recall-safe fallback (i16 factored variant:
`vqtbl2q` codebook dequant, i16 rotated query, `vmull_s16`) exists if it
is not.

## Scope

- A/B `ec_hnsw.turboquant_exact_score_mode` `exact` (current default) vs
  `int8_approx` on the standard HNSW TQ fixture at 10k/50k/100k
  (recall + latency + storage, `ecaz bench suite`, task87 counters).
- If the dip exceeds noise at 50k/100k, implement and measure the i16
  factored variant before concluding (mirrors the Task 136 fallback plan).

## Out of Scope (hard)

- No new on-disk format/mode/reloption.
- No IVF changes (Task 139 owns the IVF defaults).

## Gate / Exit Criteria

- Default flip of the HNSW exact-score mode with 10k/50k/100k evidence at
  recall within noise, or a source-grounded negative (HNSW dip is real and
  the i16 variant does not close it / erases the latency win).
