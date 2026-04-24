# Review Request: Parallel Scan N=2 No-Local-Fallback Gate

Current head: `1db8ace`

Scope:
- `src/am/ec_hnsw/scan_debug.rs`
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged PG18 `n=2` round-robin gates already checked:
  - byte-identical combined output versus serial
  - both workers contribute output
  - hidden slots, blocker metadata, and active result state all clean up after
    drain
- That still left one behavior-level blind spot: a staged pass could satisfy
  all of those end-state checks while still getting there by falling back to
  direct local-only emit along the way.
- For the current branch, that is exactly the kind of false-positive pass we do
  not want. The whole point of the remaining Task 18 work is to keep these
  cases on the shared handoff path rather than silently succeeding via local
  fallback.

What changed:
- Widened the staged round-robin debug helper so it now also returns each
  worker's `TqExplainCounters`.
- Threaded those counters through the two staged PG18 `n=2` regressions.
- Tightened both fixtures again:
  - the unique-row fixture now requires both workers to keep
    `stats_parallel_local_only_emits == 0`
  - and also `stats_parallel_deferred_local_emits == 0`
  - the duplicate-drain fixture now requires the same
- Updated the Task 18 notes to record that the staged `n=2` gates now reject
  local-only fallback emits too.

Why this matters:
- This turns the staged PG18 `n=2` coverage into a stronger shared-path
  contract: on these fixtures, matching serial output is no longer enough if
  the scan only got there by bailing out into direct local emit.
- It gives the branch a direct regression signal for one of the main ownership
  failure modes the remaining Task 18 work is trying to eliminate.
- The round-robin helper now carries enough counter context to explain that
  failure immediately if a future slice regresses.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely live-blocked
  unique outputs
- planner-visible enablement and `amcanparallel = true`
- broader `n=4/8` correctness and measurement once the final ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether “no local-only or deferred-local fallback emits on the staged PG18
  `n=2` fixtures” is the right shared-path contract at this point in Task 18
- Whether widening the round-robin helper with per-worker explain counters is a
  reasonable addition to the staged debugging surface
