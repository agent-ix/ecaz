# Review Request: Parallel Scan N=2 Round-Robin Ownership Checks

Current head: `dc6d85a`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `src/am/ec_hnsw/scan_debug.rs`
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged coordinator/hidden-slot seam had accumulated enough ownership
  plumbing that unit tests were no longer the right blocker surface.
- The first real `n=2` PG18 round-robin regression exposed two issues:
  - immediate cross-worker duplicate rows could re-emit
  - the earlier per-worker bootstrap-tail diversification could truncate the
    combined stream to the shared prefix instead of preserving serial order
- The new duplicate suppression draft also regressed hidden-owner wakeup by
  suppressing an entire element after a foreign worker drained only its first
  duplicate heap TID.

What changed:
- Added a real staged `n=2` PG18 round-robin debug harness:
  - binds two scans to the same parallel DSM allocation
  - alternates `amgettuple` calls across both workers
  - captures per-worker streams, snapshots, and visited/emitted sets
- Added a PG test that asserts the combined `n=2` round-robin stream stays
  byte-identical to the serial ordered scan.
- Worker snapshots now publish the last emitted heap TID, and duplicate
  suppression now keys on that heap TID instead of only the element TID.
- Hidden-owner wakeup now reconciles hidden-slot progress before applying that
  recent-emission suppression, so a foreign partial drain advances to the next
  duplicate instead of suppressing the whole element.
- The temporary bootstrap-tail diversification experiment was backed out.
  Parallel bootstrap workers now keep the same candidate ordering and rely on
  the shared merge/drain seam for output ordering.
- Added focused regressions proving:
  - hidden-owner wakeup clears rows fully drained by a foreign worker
  - hidden-owner wakeup advances after a foreign partial hidden drain
  - a blocked deferred row drained by a foreign worker clears from the owner's
    deferred stash on retry

Why this matters:
- This slice replaces inference with a real `n=2` correctness gate.
- The branch now has a PG18 regression that catches staged ownership mistakes
  in the actual two-worker drain order, not just unit seams around snapshots.
- It narrows the remaining blocker from “parallel ownership behavior is only
  indirectly tested” to the still-missing final contract for genuinely
  live-blocked unique outputs.

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
- Whether heap-TID-based recent-emission suppression is the right ownership key
  for duplicate rows across workers
- Whether hidden-owner wakeup now distinguishes “foreign drained one duplicate”
  from “foreign drained the whole row” correctly
- Whether keeping shared bootstrap candidate order is the right staged contract
  for preserving the `n=2` serial-equivalent stream until true traversal/work
  ownership lands
