# Review Request: Parallel Scan LWLock Release Path

Current head: `952699c`

Scope:
- `src/am/common/parallel.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The runtime `release_parallel_scan_lwlock(...)` path still gated
  `LWLockRelease(lock)` behind `InterruptHoldoffCount > 0`.
- That was intended as an abort/unwind defense, but it made ordinary release
  contingent on unrelated holdoff state and risked silently skipping a
  legitimate release if that state changed unexpectedly between acquire and
  drop.

What changed:
- Removed the `InterruptHoldoffCount` guard from the runtime LWLock release
  path.
- Runtime release now mirrors PostgreSQL's normal unconditional
  `LWLockRelease(lock)` behavior.
- Added an inline comment documenting the intended abort-path behavior:
  ordinary scope exit releases directly, while error cleanup relies on
  PostgreSQL's `LWLockReleaseAll()`.
- Updated the Task 18 note to match the new release-path contract.

Why this matters:
- It resolves the remaining release-path concern from reviewer packet 511
  before `amcanparallel = true`.
- The runtime path now follows the normal PostgreSQL locking idiom instead of
  carrying a local conditional release rule.

Still intentionally deferred:
- PG-level concurrent/ereport lock-semantics coverage before
  `amcanparallel = true`
- the broader multi-worker output ownership contract
- planner-visible parallel execution and `amcanparallel = true`

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the unconditional runtime `LWLockRelease` is the right final release
  contract for the staged coordinator serializer
- Whether the inline comment and Task 18 note describe the abort-path cleanup
  expectations clearly enough
