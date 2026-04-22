# Review Request: Parallel Scan Deferred Score Preference

Current head: `f24f443`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Deferred blocked rows were only reconsidered after the active scan phase
  finished.
- That meant a better deferred row could sit in the stash while a worse live
  local row emitted first, even though the deferred row already had the better
  score.

What changed:
- Added a score-order preference seam for deferred blocked rows.
- The scan now checks the best deferred row against the currently active local
  row before continuing the normal phase-specific emit path.
- When the deferred row already scores better, the scan drains that deferred row
  first instead of waiting for phase exhaustion.
- Refactored the deferred drain core so the non-FFI take path is available to
  unit tests:
  - `take_next_deferred_parallel_blocked_output(...)`
  - `take_preferred_deferred_parallel_blocked_output(...)`
- Added focused coverage for:
  - identifying when a deferred row should outrank the active local row
  - taking the better deferred row first while leaving the active local row
    intact for the next turn

Why this matters:
- It narrows ordering drift without pretending the final ownership-transfer
  contract is solved.
- The deferred stash now behaves more like an ordered backlog and less like a
  phase-end spill bucket.

Still intentionally deferred:
- full cross-worker ownership transfer instead of scan-local deferred fallback
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement after the remaining ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether preferring a better deferred row ahead of a worse active local row is
  the right staged ordering contract
- Whether the extracted non-FFI deferred-take helpers are the right boundary
  for unit coverage while the scan-emission path still uses Postgres FFI
