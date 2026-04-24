# Review Request: Parallel Scan Deferred Post-Handoff Progress

Current head: `7fa4706`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The deferred-blocked row path could drain a foreign handoff output and then
  lose the local cursor progress needed for the same deferred row to continue
  through the shared path.
- The selected-to-admitted blocker transition also had a stale-copy hazard:
  admission does not immediately clear the selected slot, so a retained blocker
  could drain the admitted copy and then see the same foreign output again via
  the stale selected-pending copy.
- Those gaps are exactly the ownership-transfer failures that can make the
  staged `n=2` gates pass only because later fallback behavior papers over the
  problem.

What changed:
- Added focused deferred-blocker coverage for both post-handoff progress cases:
  - a deferred row skips a local duplicate already owned by the foreign handoff,
    then drains its remaining local heap TID through shared drain
  - a retained selected blocker follows the same foreign row into the admitted
    head, drains the admitted copy, then retries local shared drain
- Advanced deferred pending cursors past heap TIDs that were just emitted by a
  foreign handoff, so the next retry cannot resurrect that heap TID locally.
- Refreshed retained selected/admitted blockers from the authoritative admission
  snapshot plus admitted result slot, instead of relying only on admitted-head
  fast-path state.
- When an admitted-head handoff drains a foreign source, cleaned up a matching
  stale selected-pending source slot so the same foreign heap TID cannot be
  returned a second time.
- Updated the Task 18 notes to record that deferred rows now prove progress
  after blocker handoff.

Why this matters:
- Deferred blocked rows now exercise both halves of the ownership transfer:
  first drain the live foreign blocker, then keep the local row on the shared
  path for its own output.
- The tests explicitly require `stats_parallel_deferred_local_emits == 0`, so
  this is not a direct-local fallback pass.
- The selected-to-admitted fixture now verifies that draining the admitted copy
  actually clears the admitted window and does not leave a stale selected copy
  to duplicate the foreign output.

Still intentionally deferred:
- planner-visible enablement and `amcanparallel = true`
- broader `n=4/8` correctness and measurement once the final ownership seam is
  ready
- final end-to-end benchmarking of live parallel plans

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the admitted-head handoff cleanup is the right layer to consume the
  stale selected-pending source copy.
- Whether the deferred cursor skip should stay heap-TID based, or whether this
  seam needs a narrower source/kind guard before planner-visible enablement.
