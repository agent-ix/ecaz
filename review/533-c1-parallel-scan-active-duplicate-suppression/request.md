# Review Request: Parallel Scan Active Duplicate Suppression

Current head: `a358159`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The branch already suppressed live-foreign duplicate heap TIDs at the
  deferred local-emission seam, but one earlier hazard remained: an active
  blocked local row could still hit a foreign selected/admitted blocker on the
  same next heap TID, fail the shared retry, and only then get deferred.
- That meant an obvious duplicate could survive longer than necessary and leak
  extra blocker churn into the deferred stash before the shared seam retried.

What changed:
- Added `live_foreign_blocker_heap_tid(...)` to read the currently live foreign
  selected/admitted heap TID for one blocker generation.
- `blocked_parallel_scan_disposition(...)` now checks the active row's next
  pending heap TID against that live foreign heap TID immediately after owner
  reconciliation.
- If the next local heap TID is already foreign-owned, the active row consumes
  that duplicate immediately, republishes its worker snapshot, clears the
  transient blocker, and:
  - returns `RetryShared` when more local pending heap TIDs remain
  - returns `DropAndContinue` when that duplicate was the last local pending
    heap TID
- The deferred duplicate-suppression helper now reuses the same shared-state
  reader.
- Added focused unit coverage for both active-row outcomes: retry-after-consume
  and drop-after-last-duplicate.
- Updated Task 18 notes to record that duplicate suppression now happens before
  defer as well as during deferred local drain.

Why this matters:
- It narrows the remaining ownership gap without claiming the final transfer
  contract is done.
- Obvious foreign-owned duplicate heap TIDs now get consumed at the first
  blocked-owner seam instead of being carried forward into deferred state.
- That keeps the remaining blocker surface concentrated on genuinely blocked
  unique outputs.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique deferred outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining ownership seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the active-row duplicate check belongs in
  `blocked_parallel_scan_disposition(...)` rather than being delayed to the
  deferred drain seam
- Whether reusing one shared-state reader for active and deferred duplicate
  suppression keeps the blocker-generation contract coherent
