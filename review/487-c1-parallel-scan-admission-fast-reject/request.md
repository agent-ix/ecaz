# Review Request: Parallel Scan Admission Fast Reject

Current head: `d9d61e8`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The new admission probe can already tell whether the selected pending output
  would enter the admitted window, but the mutating admission helper still
  took the coordinator serializer even for obvious rejections.
- That kept duplicate and full-window loser paths on the lock even when the
  probe state was still current.

What changed:
- Added `coordinator_admission_probe_is_current(...)` to cheaply validate that
  the optimistic admission probe still matches the live coordinator snapshot.
- Upgraded `admit_parallel_scan_selected_pending_output(...)` so it now:
  - reads the admission probe first
  - returns directly from that probe when `would_admit = false` and the
    coordinator state is unchanged
  - falls back to the existing locked revalidation path for actual admissions
    or stale probe state
- Added focused zero-limit coverage so the new fast-reject branch is pinned
  explicitly instead of only through duplicate/full-window cases.
- Updated Task 18 notes to record the new admission fast-reject staging seam.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no planner-visible parallel execution yet
- no final shared top-K drain path yet
- the coordinator serializer is still the staged raw lock word rather than the
  later LWLock follow-up

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 17`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether comparing the full coordinator snapshot is the right narrow validity
  check before taking the fast-reject path
- Whether the zero-limit case belongs in the same staged seam as duplicate and
  full-window loser rejection
- Whether this is the right last step before wiring the actual coordinator
  consume/admit merge loop
