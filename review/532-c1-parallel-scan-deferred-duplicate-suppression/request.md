# Review Request: Parallel Scan Deferred Duplicate Suppression

Current head: `bb5d362`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The branch still had one bad staged fallback inside deferred local emit:
  if a deferred blocked row stayed blocked and its next pending heap TID was
  already owned by a still-live foreign selected/admitted output, the local
  deferred fallback could re-emit that duplicate heap TID instead of skipping
  to the next unique local heap TID.
- That was narrower than the full ownership-transfer gap, but still a real
  duplicate hazard in the last-resort local path.

What changed:
- Added a narrow helper,
  `deferred_parallel_blocked_output_duplicates_live_foreign_heap_tid(...)`,
  that checks the deferred row's next pending heap TID against live shared
  foreign selected/admitted output snapshots.
- The deferred local fallback loop now consumes and skips that duplicate heap
  TID before deciding whether to locally emit.
- Added focused unit coverage for the helper against a live foreign selected
  pending output.
- Updated Task 18 notes to record this duplicate-suppression seam.

Why this matters:
- It removes one more concrete duplicate-output hazard from the staged local
-fallback path without pretending the full cross-worker ownership-transfer
-contract is done.
- The remaining gap is now more purely about ownership transfer for genuinely
  blocked unique outputs, not easy duplicate re-emits.

Still intentionally deferred:
- final cross-worker ownership transfer instead of deferred local retention
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement after the remaining ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the duplicate check belongs exactly at the deferred local-emission
  seam rather than earlier in the shared handoff path
- Whether comparing against the deferred row's next pending heap TID is the
  right contract for suppressing duplicate local emits
