# Review Request: Parallel Scan Hidden Owner Wakeup Reconcile

Current head: `1c674ec`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- After the previous slice, a foreign worker could correctly drain a better
  hidden owner row through the shared hidden-slot handoff path.
- But the original owner still woke up through `parallel_local_only_output_active`
  with stale local state and could republish that already-consumed hidden row.
- That left a duplicate-risk hole precisely at the hidden-owner wakeup seam.

What changed:
- Added `reconcile_parallel_hidden_owner_progress_from_shared_slot(...)`.
- Hidden-owner wakeup now checks the owner's hidden DSM slot before republishing:
  - if the hidden slot is gone, the local stale row clears
  - if the hidden slot advanced within the same element, the local duplicate
    cursor advances to match
- `try_take_parallel_scan_next_output(...)` now runs that hidden-owner
  reconciliation before it clears the local-only flag and republishes.
- Added a focused regression proving:
  - a foreign worker can drain the owner's hidden row
  - the owner's next wakeup returns `Empty` and clears its stale hidden state
    instead of resurrecting that consumed row
- Updated Task 18 notes to record the hidden-owner wakeup reconcile seam.

Why this matters:
- This closes the stale-owner side of the hidden-row handoff work from the
  previous checkpoint.
- Hidden foreign handoff is no longer enough by itself; the original owner now
  also observes that foreign progress before it republishes.
- That narrows the remaining ownership gap to the still-harder case of truly
  blocked unique rows that need cross-worker takeover, rather than stale hidden
  row resurrection.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for blocked unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the ownership contract lands

Validation:
- Passed:
  - `cargo test --lib try_take_republished_local_only_parallel_output_clears_hidden_row_drained_by_foreign_worker -- --nocapture`
  - `cargo test --lib try_take_parallel_scan_next_output_drains_better_foreign_hidden_row -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether hidden-owner wakeup now correctly distinguishes "still hidden and
  live" from "already consumed elsewhere" before it republishes
- Whether the reconcile logic can advance the local duplicate cursor without
  reviving already-consumed hidden output
- Whether this change is narrowly scoped to hidden-owner wakeup, not the normal
  published owner progress reconcile path
