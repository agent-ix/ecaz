# Review Request: Parallel Scan Active Stash Deferred Priority

Current head: `548d021`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- When an active row hit `KeepLocalEmit`, the scan already stashed that blocked
  row into the deferred blocked-output stash.
- But after stashing, both graph traversal and linear fallback immediately went
  back to fresh local work.
- That meant a better ready deferred row could still wait an extra turn even
  though the blocked active row was already out of the way.

What changed:
- After `stash_active_parallel_blocked_output(...)`, both:
  - graph traversal
  - linear fallback
  now immediately re-check `try_emit_preferred_deferred_parallel_blocked_output(...)`
  before returning to fresh local work.
- Added focused coverage proving:
  - a blocked active row stashes successfully
  - a better ready deferred row can emit first right after that stash
  - the stashed blocked row remains deferred with blocker metadata intact
- Updated Task 18 notes to record that the active `KeepLocalEmit` stash point
  now reopens deferred priority immediately.

Why this matters:
- Active blocked rows now follow the same deferred-priority rule that hidden
  local-only restash already uses.
- That keeps the staged shared/deferred ordering path tighter and reduces
  unnecessary fresh-work detours while the final ownership-transfer contract is
  still deferred.

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
- Whether the active `KeepLocalEmit` stash branches are the right places to
  reopen deferred priority immediately
- Whether any remaining stash path still returns to fresh local work before
  giving ready deferred output the same opportunity
