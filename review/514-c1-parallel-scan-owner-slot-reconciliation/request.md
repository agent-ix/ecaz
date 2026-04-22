# Review Request: Parallel Scan Owner-Slot Reconciliation

Current head: `07f5846`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Foreign selected-pending handoff is live, but the owning worker's local
  `ScanResultState` could still lag behind the shared result slot after another
  worker drained that slot through the coordinator path.
- That left a stale local duplicate-drain cursor in place and kept the blocked
  owner path falling back toward local emit even though the shared owner slot
  had already advanced or fully drained.

What changed:
- Added `reconcile_parallel_owner_progress_from_shared_slot(...)` on the scan
  side.
- Before a stable foreign-owner blocker falls back to `KeepLocalEmit`,
  `blocked_parallel_scan_disposition(...)` now reads the owner's shared result
  slot and reconciles local state when:
  - the same element is still staged but the shared `pending_index` advanced
  - the shared owner slot has already been fully drained and cleared
- When shared progress is detected, the local worker:
  - advances or clears its local duplicate-drain cursor to match the shared slot
  - republishes its worker snapshot
  - retries the shared seam instead of degrading into stale local emit
- Added focused regressions for:
  - partial shared advance of the owner slot
  - full shared drain/clear of the owner slot
- Task 18 notes now record the owner-slot reconciliation seam explicitly.

Why this matters:
- It closes the most immediate stale-owner duplicate hazard introduced by
  foreign handoff: once another worker drains the owner slot, the owner can now
  catch up to shared state before deciding to emit locally.
- This is still not full ownership transfer, but it narrows the remaining gap
  to the broader worker/consumer contract rather than leaving obviously stale
  local cursors behind.

Still intentionally deferred:
- full worker-to-worker ownership transfer beyond staged handoff and
  reconciliation
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the full multi-worker path lands
- the LWLock release-path follow-up noted in packet 511

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether shared-slot reconciliation is the right staged boundary for stale
  owner cursors before full ownership transfer lands
- Whether the local state transitions are correct for both partial shared
  advance and full shared drain of the owner slot
