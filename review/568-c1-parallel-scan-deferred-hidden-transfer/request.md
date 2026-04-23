# Review Request: Parallel Scan Deferred Hidden Transfer

Current head: `5d64f6b`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Hidden active local-only rows were already transferable through the shared
  hidden-slot blocker path.
- But once a blocked row was restashed into the deferred queue, it disappeared
  from the coordinator result slots and stayed visible only in the worker
  snapshot.
- That meant foreign workers still could not take a better ready deferred row,
  and the original owner would also keep a stale deferred stash entry if some
  later hidden handoff drained that row elsewhere.

What changed:
- `publish_parallel_scan_worker_slot_snapshot(...)` now publishes the current
  best visible deferred row into the hidden coordinator slot when there is no
  active local row, instead of clearing the slot outright.
- `try_take_parallel_scan_deferred_handoff_output(...)` now restores the best
  deferred row through the hidden local-only wakeup path so the existing
  hidden-owner reconcile logic can clear stale deferred state if another worker
  already drained that row.
- Added focused regressions proving:
  - a worse foreign worker can take a better ready deferred row through the
    shared seam
  - the original owner clears its stale deferred stash entry on the next retry
    after a foreign worker drains that row
- Updated Task 18 notes to record that best deferred rows now enter the hidden
  shared seam.

Why this matters:
- This is a concrete step on the actual remaining ownership-transfer blocker.
- Deferred rows are no longer trapped in worker-local state; the best deferred
  row can now participate in the same hidden-slot ownership transfer path as
  active hidden local-only rows.
- It narrows the remaining gap from “deferred rows are unshareable” to the
  harder contract questions around truly live blocked unique outputs.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs that are still live-blocked rather than ready
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the ownership contract lands

Validation:
- Passed:
  - `cargo test --lib try_take_parallel_scan_next_output_can_take_better_foreign_deferred_row -- --nocapture`
  - `cargo test --lib take_preferred_deferred_parallel_blocked_output_clears_foreign_drained_best_row -- --nocapture`
  - `cargo test --lib try_take_republished_local_only_parallel_output -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether publishing the best deferred row into the hidden coordinator slot is
  correctly scoped to “no active row, but deferred work exists”
- Whether the deferred restore path now reuses the hidden-owner reconcile seam
  correctly when a foreign worker has already drained that deferred row
- Whether this closes deferred-row invisibility without regressing the existing
  active hidden-row handoff behavior
