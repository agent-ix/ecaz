# Review Request: Parallel Scan Ready Deferred Shared Retry

Current head: `cc40b4c`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The deferred stash already retried the shared seam for rows that still carried
  a retained foreign blocker.
- But once a deferred row no longer carried a live blocker, the stash drained it
  directly through deferred local emit instead of first re-entering the shared
  next-output seam.
- That meant a ready deferred row could still bypass the staged coordinator
  contract even though the branch already had the machinery to restore that row
  into the shared path.

What changed:
- `take_next_deferred_parallel_blocked_output(...)` now retries
  `try_take_parallel_scan_deferred_handoff_output(...)` for any live deferred
  row while a parallel descriptor is bound, not only for rows with a retained
  blocker.
- Ready deferred rows therefore restore themselves into the shared
  next-output/owned-take path before the code ever considers deferred local
  fallback.
- Added focused coverage proving:
  - a ready deferred row drains successfully
  - the deferred-local-emit EXPLAIN counter stays at zero
  - the deferred row remains stashed with its pending cursor advanced
  - the caller's exhausted phase is restored after the shared retry
- Updated Task 18 notes to record that ready deferred rows now retry the shared
  seam before local fallback.

Why this matters:
- It narrows the remaining ownership gap without pretending the final
  cross-worker transfer contract is complete.
- Ready deferred work now stays under the staged coordinator path instead of
  slipping into local-only behavior purely because its blocker already cleared.
- That keeps the last-resort local fallback more tightly focused on genuinely
  still-blocked unique rows.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether retrying the existing deferred shared-seam helper for ready rows is
  the right staging boundary, versus introducing a separate ready-only helper
- Whether the new regression is enough proof that this path no longer falls
  through deferred local emit when the row is already ready
