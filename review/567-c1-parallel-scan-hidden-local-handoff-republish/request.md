# Review Request: Parallel Scan Hidden Local Handoff Republish

Current head: `17f7930`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Hidden local-only rows already suppressed duplicate heap TIDs against a live
  foreign owner before locally emitting again.
- But if that suppression path immediately handed off the foreign row through
  the shared seam, the owner's local duplicate cursor advanced only in local
  memory.
- The hidden DSM slot stayed absent, so the coordinator could no longer see the
  still-staged owner row and later wakeups could lose or mis-stage that hidden
  progress.

What changed:
- `try_take_parallel_scan_handoff_output(...)` now preserves
  `parallel_local_only_output_active` when the caller still has a live hidden
  row staged after emitting a foreign handoff output.
- In that case it republishes the worker snapshot and hidden coordinator slot
  directly instead of clearing local-only state and running the normal visible
  sync path.
- Strengthened
  `resolve_local_only_parallel_scan_duplicate_handoffs_live_foreign_duplicate`
  to prove the shared hidden slot advances to the owner's next local heap TID
  after the duplicate is suppressed and the foreign row is handed off.
- Updated Task 18 notes to record that hidden duplicate suppression now
  republishes the hidden row before the handoff returns.

Why this matters:
- This closes a real ownership bookkeeping hole, not just a missing test.
- After a foreign handoff on the duplicate-suppression path, the owner's hidden
  DSM slot now stays aligned with its local duplicate cursor.
- That keeps hidden-owner state visible to the remaining ownership-transfer
  machinery instead of silently dropping the row out of shared state.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the ownership contract lands

Validation:
- Passed:
  - `cargo test --lib resolve_local_only_parallel_scan_duplicate_handoffs_live_foreign_duplicate -- --nocapture`
  - `cargo test --lib try_take_republished_local_only_parallel_output -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether preserving hidden local-only state in
  `try_take_parallel_scan_handoff_output(...)` is correctly scoped to callers
  that still own a hidden staged row
- Whether the strengthened regression proves the shared hidden slot now tracks
  the advanced local duplicate cursor instead of disappearing
- Whether the direct hidden republish path avoids accidentally running the
  visible-owner reconcile logic on hidden rows
