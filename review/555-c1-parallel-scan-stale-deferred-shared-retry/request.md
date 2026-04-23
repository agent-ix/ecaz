# Review Request: Parallel Scan Stale Deferred Shared Retry

Current head: `6ee5d12`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Deferred rows whose retained blocker went stale during the
  `allow_local_emit = true` drain pass were clearing the stale blocker, but
  then still dropping straight into the local-emission loop.
- That meant a row that had become ready again late in deferred drain could
  bypass one more shared next-output retry and fall back to direct local emit.

What changed:
- In the deferred local-emission loop, once a retained blocker clears stale,
  the row now gets one more `try_take_parallel_scan_deferred_handoff_output(...)`
  retry before the branch considers direct local emit.
- Strengthened the stale-blocker deferred regression to prove this is a real
  shared retry, not just an uncounted local emit:
  - the deferred row now has two heap tids
  - the test records coordinator snapshot generations before and after
  - it asserts `result_publish_generation` advanced
  - it asserts the row stayed deferred with its pending cursor advanced and
    no retained blocker
- Updated Task 18 notes to record that stale deferred blockers now re-enter the
  shared seam even inside the late deferred drain pass.

Why this matters:
- Stale-blocker rows stay on the staged shared path longer instead of peeling
- off into local fallback as soon as the drain pass reaches them.
- This narrows the remaining ownership-transfer gap without pretending the full
  cross-worker protocol is done.
- The new regression distinguishes real shared retry from merely “not counted
  as local fallback.”

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo fmt`
  - `cargo test take_next_deferred_parallel_blocked_output_clears_stale_blocker_before_emit -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether stale deferred blockers now get a real shared retry in the late drain
  pass instead of only clearing metadata before local emit
- Whether the strengthened regression proves shared coordinator activity, not
  just the absence of deferred local-emit counters
