# Review Request: Parallel Scan Deferred Worker Snapshot Visibility

Current head: `9c96880`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- Once a worker had no live active-result cursor, its shared runtime snapshot
  looked idle even if the worker still held a best deferred blocked row.
- That hid the remaining local ownership state from diagnostics and made the
  blocker snapshots harder to interpret while the final cross-worker transfer
  seam is still pending.

What changed:
- Added `published_parallel_worker_result_state(...)` to choose the result state
  that should be visible in the shared worker snapshot.
- Worker snapshot publication now prefers the active result when it has a live
  current row, otherwise it falls back to the best deferred blocked row.
- The published worker phase, pending duplicate count, and current-row bit now
  follow that visible deferred row instead of defaulting to an apparently idle
  worker state.
- Extended the deferred-blocker snapshot regression to verify:
  - the published phase matches the best deferred blocked row
  - the worker snapshot still reports a live current row
  - the pending duplicate count reflects the deferred row state

Why this matters:
- It makes the staged ownership/blocker diagnostics truthful even before the
  final handoff protocol lands.
- Reviewers can now distinguish “worker is actually idle” from “worker is
  blocked but still holding a deferred row”.

Still intentionally deferred:
- full cross-worker ownership transfer instead of deferred local retention
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement after the remaining ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Note:
  - an earlier overlapping `cargo test` run fanned out into the known broad
    `pgrx` harness failure mode while `ecaz dev test pgrx --pg 18` was already
    active; the serial `cargo test` rerun on the unchanged tree was green and
    is the result that counts for this checkpoint

Review focus:
- Whether showing the best deferred blocked row in the shared worker snapshot is
  the right staged visibility contract before ownership transfer lands
- Whether limiting this slice to diagnostic visibility, without changing
  coordinator slot publication, is the right boundary
