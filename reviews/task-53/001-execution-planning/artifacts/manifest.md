# Task 53 / 001 — Execution Planning · Artifact Manifest

Packet path: `reviews/task-53/001-execution-planning/`
Task: `plan/tasks/53-common-p6-datum-wrappers.md`
Branch: `task-53` (off `origin/main` `5c0e9e2bd`).
Head SHA at packet authoring: `5c0e9e2bd` (Task 52 lives separately on
`task-52` branch, not yet merged into main).

## Surfaces

This is a planning packet — no code change under review. Pre-state
baseline + consumer-site survey to anchor the slice plan and the
closeout's before/after delta math.

## Artifacts

### `baseline-unsafe-density.txt`
- Per-line census of all 29 `unsafe { ... }` blocks in
  `src/am/ec_hnsw/source.rs`.
- SIMD subtotal (10 blocks, out of scope per §Non-Goals) separated
  from datum-handling subtotal (19 blocks, in scope).
- `src/` total: 960.
- HNSW subsystem total at branch start: 327 (matching Task 50 close).
- Note that `task-52`'s wrapper modules (`dsm.rs`,
  `parallel_context.rs`) are NOT present on this branch since Task 52
  hasn't been merged to main yet.
- Command: `grep -nE "unsafe\s*\{" src/am/ec_hnsw/source.rs`.
- Timestamp: 2026-05-23.

### `source-rs-consumer-survey.txt`
- Survey of existing wrappers in `src/am/common/detoast.rs` (already
  lifted: `DetoastedVarlena`) and HNSW-local wrappers in source.rs
  (candidates for lift to `src/am/common/datum.rs`:
  `DetoastedFloat4Datum`, `FlatFloat4ArrayRef`,
  `FlatFloat4VarlenaRef`, `FlatFloat4SourceRef`).
- Per-line mapping of each datum-handling unsafe block to the wrapper
  it'll be absorbed into.
- Estimated consumer-side reduction: -16 to -21 blocks → source.rs
  29 → 8-13 (target ≤14 is tractable).
- Command refs used to build the survey.
- Timestamp: 2026-05-23.

## What this packet does not include

- No code change, no test logs, no bench evidence. Subsequent slice
  packets (002+) carry those.
- No HNSW subsystem migration beyond source.rs in this task. SPIRE /
  IVF / DiskANN consumer migrations of the new wrappers are deferred
  to Tasks 55/56/57 per task spec §Non-Goals and §Coordination.

## §Exit Criteria status (planning baseline)

| # | Criterion | Status |
| - | --- | :-: |
| 1 | Four typed wrappers in `src/am/common/datum.rs` | not started |
| 2 | `src/am/ec_hnsw/source.rs` ≤ 14 | not started (current: 29) |
| 3 | HNSW recall + QPS + per-row storage no regression | not started |
| 4 | Closing summary packet with deltas + handoff list | not started |
