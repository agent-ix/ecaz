# Review Request: Parallel Scan N=2 Active-State Cleanup Gate

Current head: `5a1116d`

Scope:
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged PG18 `n=2` round-robin gates already required:
  - byte-identical combined output versus serial
  - both workers to contribute output
  - both hidden local-only coordinator slots to be empty after drain
  - both worker blocker-kind fields to be clear after drain
- That still left one more cleanup blind spot: a staged pass could clear the
  shared hidden slots and blocker metadata while still leaving a live current
  row or pending duplicate cursor behind in the worker runtime snapshot.
- For the current branch, that is still not “done.” If a worker snapshot says
  it still has a current row or pending output after the fixture has fully
  drained, the staged ownership cleanup is incomplete even if the merged stream
  already matched serial.

What changed:
- Tightened both staged PG18 `n=2` round-robin regressions again:
  - the unique-row fixture now also requires both workers to clear
    `active_result_has_current` and `active_result_pending_count`
  - the coalesced-duplicate fixture now requires the same active-state cleanup
- Kept the existing output, contribution, hidden-slot, and blocker cleanup
  assertions, then layered the worker active-state cleanup assertion on top.
- Expanded the failure output so a future regression still prints the worker
  snapshots, hidden snapshots, streams, visited sets, and emitted sets when
  active-state cleanup fails.
- Updated the Task 18 notes to record that the staged `n=2` gates now reject
  stranded active result state too.

Why this matters:
- This turns the staged PG18 `n=2` coverage into a stronger exhaustion
  contract: once the fixture drains, the worker snapshots must also agree that
  no current row or pending duplicate cursor is still live.
- It closes another false-positive path where the merged stream can look right
  while worker-local runtime cleanup is still incomplete.
- That keeps the staged gates aligned with the branch’s remaining ownership
  work instead of only validating final output order.

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
- Whether “worker snapshots must clear current-row and pending-count state
  after the stream drains” is the right additional staged cleanup contract for
  the PG18 `n=2` fixtures
- Whether this strengthens the ownership gate usefully without over-constraining
  the final implementation
