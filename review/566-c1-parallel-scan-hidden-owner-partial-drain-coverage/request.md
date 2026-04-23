# Review Request: Parallel Scan Hidden Owner Partial-Drain Coverage

Current head: `cf27bbd`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- The previous hidden-owner wakeup reconcile slice proved the all-or-nothing
  case where a foreign worker drained the owner's hidden row completely before
  the owner woke again.
- But there was still no focused regression for the partial-drain case where a
  foreign worker consumes only the first hidden duplicate and the owner should
  resume at the next duplicate instead of reviving the consumed one.

What changed:
- Added a focused regression:
  - `try_take_republished_local_only_parallel_output_advances_after_foreign_partial_hidden_drain`
- The test sets up a hidden local-only owner row with two duplicate heap TIDs,
  lets a foreign worker drain the first duplicate through the shared hidden-slot
  handoff path, then verifies the owner's wakeup emits only the second
  duplicate.
- The regression also proves the owner clears its local-only wakeup flag and
  exhausts the local row after that resumed emit.

Why this matters:
- This pins the duplicate-cursor half of the hidden-owner wakeup reconcile
  logic, not just the fully drained case.
- It makes the hidden-owner handoff seam harder to regress while the remaining
  ownership-transfer work continues.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the ownership contract lands

Validation:
- Passed:
  - `cargo test --lib try_take_republished_local_only_parallel_output_advances_after_foreign_partial_hidden_drain -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the new regression really exercises the shared hidden-slot handoff
  plus owner wakeup resume path, not just a local duplicate drain
- Whether the asserted postconditions correctly capture partial hidden-row
  progress after a foreign drain
