# Review Request: Parallel Scan Local-Only Duplicate Suppression

Current head: `9fa5b89`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The branch already suppressed live foreign-owned duplicate heap TIDs in the
  active blocked-owner path and in deferred blocked-output drain.
- But a row hidden behind `parallel_local_only_output_active` could still wake
  on the `Empty` owned-output path and locally emit a heap TID that a still-live
  foreign selected/admitted output already owned.
- That left one more duplicate leak in the staged ownership path, even after
  the earlier active/deferred duplicate-suppression slices landed.

What changed:
- Added `resolve_local_only_parallel_scan_duplicate(...)` to run the same
  live-foreign duplicate check against a hidden local-only row before its wakeup
  path locally emits again.
- If the next local heap TID is still owned by a live foreign selected/admitted
  output, the wakeup path now:
  - consumes that duplicate heap TID locally
  - retries shared handoff for the foreign output
  - only falls back to local emit for the next unique local heap TID
- If the duplicate was the last remaining heap TID for that hidden local-only
  row, the wakeup path now exhausts and clears the row instead of re-emitting
  the duplicate locally.
- Wired that helper into both:
  - graph-traversal local-only wakeup
  - linear-fallback local-only wakeup
- Added focused regressions proving:
  - a hidden local-only row hands off a live foreign duplicate before local emit
  - exhausting the last such duplicate clears the hidden local-only row
- Updated Task 18 notes so the local-only wakeup seam is recorded alongside the
  earlier active/deferred duplicate-suppression slices.

Why this matters:
- It closes another real duplicate-emission gap without claiming the final
  cross-worker ownership-transfer contract is done.
- The branch now uses the same live-foreign duplicate rule across all three
  staged ownership surfaces:
  - active blocked rows
  - deferred blocked rows
  - hidden local-only wakeup rows
- That keeps the remaining blocker focused on genuinely blocked unique outputs
  instead of duplicate leakage.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
- Note:
  - the first `ecaz dev test pgrx --pg 18` attempt failed only because I had
    overlapped it with `cargo test`, which left a stale `postmaster.pid` and
    tripped the known `pgrx` test mutex cascade
  - rerunning the PG18 pgrx lane serially on a clean harness passed

Review focus:
- Whether local-only wakeup is the right place to reuse the live-foreign
  duplicate rule, versus trying to force this case through a different shared
  ownership seam first
- Whether the two new regressions are enough proof that hidden local-only rows
  no longer re-emit foreign-owned duplicate heap TIDs
