# Review Request: Parallel Scan Deferred Duplicate Handoff Retry

Current head: `0f26809`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The branch already suppressed a deferred row's next heap TID when a live
  foreign selected/admitted output already owned that duplicate.
- But after consuming that duplicate in the deferred drain seam, the code still
  slid straight toward local-only fallback for the same row.
- That meant the worker could miss an immediate shared handoff opportunity even
  though the retained blocker still pointed at a live foreign output.

What changed:
- In `take_next_deferred_parallel_blocked_output(...)`, after deferred local
  drain consumes a live foreign-owned duplicate heap TID, the row now retries
  `try_take_parallel_scan_handoff_output(...)` immediately when:
  - local pending output still remains
  - the row still has a live current element
  - a retained blocker is present
- If that shared retry drains the foreign selected/admitted output:
  - the local deferred row is pushed back into the deferred stash when it still
    has pending output
  - the shared output is returned immediately
- If the shared retry still leaves the deferred row obsolete, it drops there
  instead of being carried farther toward local emit.
- Added focused coverage proving that:
  - after skipping the duplicate heap TID, deferred drain re-enters the shared
    handoff path first
  - the local deferred row remains stashed afterward for later ownership
    resolution
- Updated Task 18 notes to record that deferred duplicate skip now reopens the
  shared handoff seam.

Why this matters:
- It closes another remaining "local fallback too early" gap without claiming
  the final ownership-transfer contract is done.
- Duplicate suppression in deferred drain now does the next obvious thing:
  consume the foreign-owned duplicate, then retry the shared seam immediately.
- That keeps the remaining blocker surface focused on genuinely unique outputs
  that are still foreign-owned, not on rows that merely needed one more shared
  retry after duplicate suppression.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique deferred outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining ownership seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether retrying the shared handoff directly from the deferred duplicate-skip
  seam is the right narrow step before the full ownership-transfer contract
- Whether the deferred row's retained blocker and staged local state are
  preserved cleanly enough after the foreign handoff drains
