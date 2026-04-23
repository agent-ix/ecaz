# Review Request: Parallel Scan Hidden-Owner Blockers

Current head: `ce96cec`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Retained selected/admitted blockers could go falsely stale once the foreign
  owner stopped publishing through the coordinator fast paths and parked that
  same row in a hidden local-only DSM slot.
- That made hidden local-only wakeups and deferred blocked rows treat
  themselves as ready too early, even though the same foreign owner still held
  the row and score/heap-tid state privately.

What changed:
- Added `read_parallel_scan_hidden_local_only_result_slot_snapshot(...)` so
  scan-side ownership logic can read a live hidden local-only worker slot
  without surfacing it to the coordinator.
- `live_foreign_blocker_heap_tid(...)` now extends blocker continuity through
  hidden local-only owner slots for:
  - `ForeignSelectedPending`
  - `ForeignAdmittedHead`
- `deferred_parallel_blocked_output_preference_score(...)` now uses the same
  hidden-slot continuity, so blocked deferred ordering keeps honoring the
  foreign owner's actual row score instead of marking the blocker stale.
- The hidden-slot reader returns `Ok(None)` for out-of-range slot indices so
  stale-blocker paths still degrade cleanly instead of turning old invalid-slot
  tests into hard errors.
- Added focused regressions proving:
  - a retained selected blocker stays live when the foreign owner moves into a
    hidden local-only slot
  - a retained admitted-head blocker keeps contributing the hidden foreign
    owner's score after that same transition
- Updated Task 18 notes to record hidden-owner blocker continuity.

Why this matters:
- This closes another false-ready gap in the staged ownership model.
- Hidden and deferred rows now keep honoring the same foreign owner while it
  moves from selected/admitted publication into hidden local-only state.
- It narrows the remaining ownership-transfer gap without claiming the final
  cross-worker contract is complete.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether retained blockers now stay live across selected/admitted to hidden
  local-only transitions for the same foreign owner row
- Whether deferred ordering now uses hidden foreign-owner score state instead
  of falsely treating those blockers as stale
- Whether the new hidden-slot continuity stays narrowly scoped to the same
  foreign owner slot/element rather than broadening into unrelated local-only
  worker state
