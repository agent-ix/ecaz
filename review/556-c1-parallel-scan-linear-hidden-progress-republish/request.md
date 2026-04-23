# Review Request: Parallel Scan Linear Hidden Progress Republish

Current head: `2fb393e`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The graph local-only wakeup path already republished worker progress after a
  direct hidden-row emit, but the linear pending-output wakeup branch did not.
- That left the worker runtime snapshot stale after a linear hidden-row emit:
  emitted-count progress and hidden-row wakeup state could lag behind the
  actual local duplicate cursor even though the coordinator slot remained
  intentionally cleared.

What changed:
- Extracted the linear hidden-row emit tail into:
  - `emit_local_only_linear_pending_output(...)`
  - `finish_local_only_linear_pending_emit(...)`
- The linear pending-output wakeup branch now republishes worker progress after
  advancing the duplicate cursor, while still keeping the coordinator result
  slot cleared under `parallel_local_only_output_active`.
- Added a focused regression that:
  - stages a hidden linear local-only row with two heap tids
  - simulates the pending-output take without Postgres FFI
  - finishes the post-emit wakeup path
  - asserts the worker snapshot reflects emitted progress
  - asserts the coordinator result slot stays cleared
- Updated Task 18 notes to record that hidden linear wakeup emits now
  republish worker progress.

Why this matters:
- Hidden linear wakeup emits now keep shared runtime diagnostics aligned with
  the local duplicate cursor instead of silently drifting after the first local
  emit.
- This makes the remaining ownership-transfer seam easier to reason about
  without broadening scope into planner enablement or the final cross-worker
  contract.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo fmt`
  - `cargo test finish_local_only_linear_pending_emit_republishes_hidden_worker_snapshot -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the linear hidden-row wakeup path now republishes worker runtime
  progress after a direct local emit without exposing the coordinator result
  slot
- Whether the new regression proves the intended hidden-row contract without
  introducing another multithreaded Postgres FFI unit test
