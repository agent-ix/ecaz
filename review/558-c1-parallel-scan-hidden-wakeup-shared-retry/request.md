# Review Request: Parallel Scan Hidden Wakeup Shared Retry

Current head: `98c468a`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Hidden local-only wakeups were still short-circuiting out of the shared
  ownership seam whenever `retained_parallel_owned_output_blocker` was present.
- That meant a hidden row would skip a real shared retry even when:
  - the retained blocker was already stale, or
  - the still-live foreign blocker could have been drained immediately through
    the shared handoff path.
- The result was unnecessary fallback pressure into deferred/local-only emit
  even though the shared seam was already capable of making progress.

What changed:
- `try_take_republished_local_only_parallel_output(...)` now only requires
  `parallel_local_only_output_active`; it no longer rejects the wakeup just
  because retained blocker metadata is still attached.
- Hidden local-only wakeups now always republish and re-read shared ownership
  first.
- Added focused regressions proving both sides of that change:
  - a hidden row with a still-live retained foreign-selected blocker now
    retries the shared seam and drains the foreign selected output instead of
    returning `Empty`
  - stale retained blocker metadata no longer prevents a hidden row from
    re-entering the shared path and consuming its own next output
- Updated Task 18 notes to record that hidden wakeups now retry shared
  ownership first before any later fallback.

Why this matters:
- This narrows the real remaining ownership gap without pretending the final
  cross-worker transfer contract is done.
- Hidden wakeups now make one more legitimate ownership/handoff attempt before
  the scan falls back to deferred or local-only behavior.
- It reduces cases where stale or merely persistent blocker metadata suppresses
  shared progress that is already available.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test try_take_republished_local_only_parallel_output_checks_live_retained_blocker -- --nocapture`
  - `cargo test try_take_republished_local_only_parallel_output_ignores_stale_retained_blocker_gate -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether hidden local-only wakeups now correctly take one real shared retry
  even when retained blocker metadata is still present
- Whether the live-blocker regression proves we can hand off the foreign
  selected row without disturbing the hidden local row's own cursor
- Whether the stale-blocker regression proves retained metadata no longer gates
  a hidden row away from the shared path
