# Task 144: HNSW int8_approx exact-score default revisit

Status: **complete - default flipped + confirmed** (2026-07-03). Flip landed
on main as `3f69d74c0` (`exact` → `auto`: int8_approx on the no-QJL 4-bit
lane, exact elsewhere). Default path confirmed in
`reviews/task-144/002-auto-default-confirm/` — auto recall matches the
explicit-int8 A/B byte-for-byte at all 18 ef cells (10k/50k/100k × ef 40–200)
and differs from exact at every point; fallback-lane smoke shows auto scans
clean where int8 is unsupported (explicit int8 errors there). Approval:
`reviews/task-144/001-hnsw-scorer-default/feedback/2026-07-03-01-reviewer.md`.
Owner: Codex. Priority: P3

Evidence: `reviews/task-144/001-hnsw-scorer-default/` — recall dips
≤0.42 pp across ef 40–200 at 10k/50k/100k (noise; Task 98 caution does
not reproduce), latency −10% at the ef64 mid points and neutral
elsewhere. Recommends flipping the HNSW mode default to int8_approx
(modest win, never worse); reviewer may prefer documented opt-in.
Follow-up to Tasks 98 (int8_approx kernel, HNSW wiring) and 136 (IVF
evidence); sweetened by Task 141 (SDOT) if it lands first.

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
- No IVF changes (Task 143 owns the IVF defaults).

## Gate / Exit Criteria

- Default flip of the HNSW exact-score mode with 10k/50k/100k evidence at
  recall within noise, or a source-grounded negative (HNSW dip is real and
  the i16 variant does not close it / erases the latency win).
