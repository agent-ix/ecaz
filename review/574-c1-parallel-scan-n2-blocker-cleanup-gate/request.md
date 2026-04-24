# Review Request: Parallel Scan N=2 Blocker Cleanup Gate

Current head: `657d8c9`

Scope:
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged PG18 `n=2` round-robin gates already required:
  - byte-identical combined output versus serial
  - both workers to contribute output
  - both hidden local-only coordinator slots to be empty after the fixture
    drains
- That still left one cleanup blind spot: a staged pass could drain the stream
  and clear the hidden slots while still leaving blocked-owner metadata behind
  in the worker runtime snapshots.
- For the current branch, that is not a harmless leftover. The remaining live
  ownership seam is driven by blocker state, so a staged pass should not count
  as healthy if the shared worker snapshots still say a worker is blocked after
  the fixture is already fully drained.

What changed:
- Tightened both staged PG18 `n=2` round-robin regressions again:
  - the unique-row fixture now also requires both workers'
    `owned_output_blocker_kind` fields to be `NONE` once the combined stream
    drains
  - the coalesced-duplicate fixture now requires the same blocker cleanup
- Kept the existing serial-stream, dual-worker contribution, and hidden-slot
  cleanup assertions, then layered the worker blocker cleanup assertion on top.
- Expanded the failure output so a future regression still prints the worker
  snapshots, hidden snapshots, streams, visited sets, and emitted sets when
  blocker cleanup fails.
- Updated the Task 18 notes to record that the staged `n=2` gates now reject
  stranded blocker metadata too.

Why this matters:
- The branch's remaining blocker is a live ownership-transfer seam. Worker
  blocker metadata is part of that seam, not just debugging garnish.
- This turns the staged PG18 `n=2` coverage into a stronger cleanup contract:
  once the fixture drains, the shared worker snapshots must also agree that no
  blocked-owner state is still pending.
- It narrows the space for false-positive staged passes before the branch moves
  deeper into the final live blocked unique-output transfer work.

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
- Whether “worker blocker-kind fields must be clear after the stream drains” is
  the right additional staged cleanup contract for the PG18 `n=2` fixtures
- Whether this assertion usefully tightens the ownership gate without
  over-constraining the final implementation
