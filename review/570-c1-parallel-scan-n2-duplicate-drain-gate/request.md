# Review Request: Parallel Scan N=2 Duplicate-Drain Gate

Current head: `9dbe6ad`

Scope:
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged `n=2` round-robin ownership gate was live, but it only covered the
  base serial-equivalent stream.
- The branch also needed an end-to-end PG18 regression that exercises
  coalesced duplicate vectors, because the recent ownership work is heavily
  about duplicate heap-TID drain across workers rather than only unique rows.
- Without that gate, the hidden/deferred duplicate-suppression seams were still
  mostly protected by unit tests instead of a real two-worker PG path.

What changed:
- Added a second real `n=2` PG18 round-robin regression:
  - builds a heap-backed duplicate fixture with coalesced vector rows
  - runs the same two-worker staged round-robin debug harness
  - asserts the combined emitted heap-TID and score stream stays byte-identical
    to the serial duplicate-drain order
- Updated the Task 18 notes to record that the staged `n=2` duplicate-drain
  gate is now live end-to-end.

Why this matters:
- This closes a gap between the ownership plumbing and the branch’s real PG18
  coverage.
- The parallel branch now has an end-to-end regression that proves duplicate
  drain stays serial-equivalent under staged two-worker execution, not just
  unique-row ordering.
- That makes the remaining blocker narrower: the open ownership seam is now the
  truly live blocked unique-output handoff contract, not unpinned duplicate
  drain behavior.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely live-blocked
  unique outputs
- planner-visible enablement and `amcanparallel = true`
- broader `n=4/8` correctness and measurement once the final contract lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the new duplicate fixture is the right end-to-end PG18 surface for
  staged parallel duplicate-drain correctness
- Whether asserting the byte-identical emitted heap-TID and score stream is the
  right contract for this branch state
- Whether any duplicate-drain failure mode still remains uncovered before the
  branch moves back to the unique-output ownership-transfer seam
