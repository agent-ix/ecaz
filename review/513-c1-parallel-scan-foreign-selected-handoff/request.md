# Review Request: Parallel Scan Foreign Selected-Pending Handoff

Current head: `7cd2288`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged scan-side handoff seam could already consume a foreign row once it
  had reached the admitted window, but it still treated foreign
  selected-pending output as a hard blocker.
- That kept the staged multi-worker seam degrading too early into blocked or
  local-only behavior even when the shared global next-output path already
  knew which foreign row should win next.

What changed:
- Broadened `try_take_parallel_scan_handoff_output(...)` so the scan side can
  hand off both:
  - `ForeignSelectedPending`
  - `ForeignAdmittedHead`
- When blocked by a foreign selected-pending row, the helper now drains the
  globally selected next output through
  `take_parallel_scan_next_output_snapshot(...)` instead of reporting a blocked
  state immediately.
- The handoff path still preserves the local worker's staged cursor:
  - local `current_result` stays intact
  - local duplicate-drain progress stays intact
  - the local staged row remains published for the next retry
- Updated the materialized and prefetched scan-side staging regressions to
  assert emitted handoff behavior instead of the old blocked fallback.
- Task 18 notes now record that foreign selected-pending handoff staging is
  live, while full ownership transfer is still deferred.

Why this matters:
- It closes the last staged handoff gap between the already-admitted path and
  the globally selected pending path.
- The remaining ownership problem is now narrower: this slice can consume the
  shared winning foreign row, but it does not yet establish a full
  worker-to-worker ownership transfer protocol for planner-visible parallel
  execution.

Still intentionally deferred:
- full foreign selected-pending ownership transfer beyond staged handoff
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether scan-side handoff of foreign selected-pending rows is the right
  staged boundary before full ownership transfer lands
- Whether the local worker state is preserved cleanly while draining the shared
  winning foreign row through the coordinator merge path
