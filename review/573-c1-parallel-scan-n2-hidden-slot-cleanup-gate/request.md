# Review Request: Parallel Scan N=2 Hidden-Slot Cleanup Gate

Current head: `ba5a9f8`

Scope:
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged PG18 `n=2` round-robin gates already checked two important
  ownership properties:
  - the combined stream still matched the serial stream
  - both worker streams contributed output
- That still left one gap in the staged ownership contract: a two-worker pass
  could preserve the final combined output and worker contribution while still
  leaving a hidden local-only coordinator slot stranded after the fixture had
  already drained.
- For the remaining parallel ownership work, that is not a harmless detail. A
  stranded hidden DSM row means shared state cleanup has already diverged from
  the staged ownership model the branch is trying to tighten.

What changed:
- Tightened both staged PG18 `n=2` round-robin regressions again:
  - the unique-row fixture now requires both workers' hidden local-only
    coordinator slot snapshots to be empty once the combined stream drains
  - the coalesced-duplicate fixture now requires the same hidden-slot cleanup
    too
- Kept the existing serial-stream and both-workers-contribute assertions, then
  layered the new hidden-slot cleanup assertion on top.
- Expanded the failure text so a future regression prints both hidden-slot
  snapshots alongside the existing worker snapshot, stream, visited, and
  emitted traces.
- Updated the Task 18 notes to record that the staged `n=2` gates now reject
  stranded hidden DSM rows.

Why this matters:
- The branch's remaining blocker is a shared ownership-transfer seam, not basic
  staged output matching.
- Hidden local-only DSM rows are part of that seam. If a staged `n=2` fixture
  finishes with hidden rows still stranded in shared state, that should count
  as a correctness failure even when the final merged stream still looks right.
- This makes the current PG18 `n=2` gates stricter about shared-state cleanup
  before the branch moves deeper into the final live-blocked ownership
  transfer work.

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
- Whether “hidden local-only coordinator slot snapshots must be empty after the
  stream drains” is the right staged cleanup contract for these PG18 `n=2`
  fixtures
- Whether this assertion tightens the current ownership gate usefully without
  over-constraining the final implementation
