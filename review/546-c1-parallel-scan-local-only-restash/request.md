# Review Request: Parallel Scan Local-only Restash

Current head: `fc068ac`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Hidden local-only rows already retried duplicate suppression and shared
  handoff on wakeup.
- But when the foreign blocker was still genuinely live afterward, the scan
  went straight back to local-only emit.
- That kept some still-blocked unique rows leaving the staged shared path
  earlier than necessary.

What changed:
- Added `restash_local_only_parallel_blocked_output(...)` so a still-blocked
  hidden local-only row moves back into the deferred blocked-output stash
  instead of immediately re-emitting locally.
- Wired that restash behavior into both:
  - graph traversal wakeup
  - linear fallback wakeup
- After restashing, graph traversal refreshes prefetch and resumes normal scan
  production; linear fallback resumes its normal result-selection loop.
- Added focused unit coverage for:
  - moving a blocked hidden local-only row into the deferred stash with blocker
    metadata intact
  - leaving a blocker-free hidden local-only row alone
- Updated Task 18 notes to record that still-blocked local-only wakeups now
  restash before returning to local-only emit.

Why this matters:
- More genuinely blocked unique rows stay on the staged deferred/shared path
  instead of falling back to local-only emit immediately on wakeup.
- This narrows the remaining ownership gap without pretending the final
  cross-worker ownership-transfer contract is landed.

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
- Whether the graph/linear wakeup control points are the right places to
  restash still-blocked hidden local-only rows
- Whether there is any remaining local-only wakeup path that should also
  restash instead of returning directly to local-only emit
