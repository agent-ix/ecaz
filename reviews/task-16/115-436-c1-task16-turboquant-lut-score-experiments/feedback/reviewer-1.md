## Feedback: TurboQuant live LUT score experiments — ACCEPTED

Verified against:

- commit `572dd53` extending
  `TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE` to four values: `exact`,
  `full_lut`, `tiled_lut`, `int8_approx`
- `src/am/scan.rs` prepared scan state now holds optional
  `PreparedLutNoQjl4BitQuery` /
  `PreparedTiledLutNoQjl4BitQuery` alongside the int8 query
- `score_scan_element_result(...)` dispatching to four scorer
  variants under the same gate
- `src/lib.rs` pg tests covering default, `full_lut`, `tiled_lut`,
  `int8_approx`, and invalid env
- `scripts/vacuum_concurrency_scratch.sh` now accepts
  `--socket-dir` / `--port`

### What's right

- **Unifies lever-4 and lever-5 experiments under one gate.** Same
  env, same seam, same dispatch site in
  `score_scan_element_result(...)`. That makes cross-mode
  measurement apples-to-apples — a reader cannot mistakenly
  conclude a difference comes from the scaffolding rather than the
  scorer.
- **All four modes prepared and freed symmetrically.** The
  prepared-query state additions mirror the existing int8 and
  binary-sign state management. No one-off lifetimes.
- **Invalid-env coverage expanded.** The env-rejection test now
  checks the expanded allowed-value list. This was the right
  place to lock the list down; packet `437` needs it.
- **Vacuum concurrency harness cluster-args.** Adding
  `--socket-dir` / `--port` to `vacuum_concurrency_scratch.sh` was
  the missing piece to run the V3 vacuum-concurrency rerun
  requested in packet `428` feedback without env-prefix
  invocation. This packet delivered that surface; packet `437`
  then ran it and surfaced a real bug. Good chain of work.
- **Default behavior unchanged.** Confirmed in the readout and in
  the default-mode pg test.

### Concerns

1. **Score-parity coverage is still label-only.** The four new
   stage-profile pg tests verify the mode label and that rerank /
   prefilter counters fire. None of them pin "result set stays
   close to exact on a small fixture." Packet `437` later shows
   recall was preserved in the matrix, but that was a measurement
   observation on the real 50k lane, not a unit-level invariant.
   A fixture-level top-K parity assertion per mode would catch a
   future silent scorer drift before it reached measurement.
2. **`tiled_lut` with a hard-coded tile size.** The scan-path
   dispatch to `score_ip_from_parts_tiled_lut_no_qjl_4bit(...)`
   does not appear to accept a runtime tile size. Packet `433`'s
   offline study used `--tile-size 512` explicitly. Worth naming
   the pinned tile size in the code (comment or const) and in
   the stage-profile output, so packet `437`'s tiled_lut numbers
   are unambiguously attributable to one tile size.
3. **Four scorer modes × two rerank modes = eight cells in packet
   `437`.** With a single experimental env, switching between
   modes requires a scratch restart (measured in packet `437`).
   Not a bug here — just a note that the env-gate shape forces
   restart-level measurements. A session-level SET would have let
   packet `437` run tighter cells, but is more invasive; the
   env-gate is defensible for an experimental seam.

### Call

Accepted. The packet correctly unifies all four scorer experiments
under one seam and ships the vacuum-concurrency harness surface
that `437` then used to surface a real bug. Please follow up on
score-parity coverage (concern `1`) before any of these modes move
beyond `TQVECTOR_*` env into persisted reloption territory.
