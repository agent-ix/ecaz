# Review Request: Parallel Scan Coordinator Fast-Path Read

Current head: `dd34d50`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The prior Task 18 slice staged the coordinator-owned snapshot of the current
  best published result slot, but callers still had to use the generic
  slot-scan selection helper to read it back.
- That left the next coordinator merge work without a direct "read the current
  coordinator-selected staged result" helper, even though the coordinator
  header already carried that state.
- We needed a fast-path read seam that trusts the coordinator snapshot first,
  validates the named result slot, and returns that current staged result
  directly.

What changed:
- Added `coordinator_result_slot_snapshot_is_live(...)` to centralize the
  validity checks for staged result slots:
  - current rescan epoch
  - published flag set
  - score-valid flag set
  - valid element TID
- Refactored the existing scan-based selection helper to reuse that predicate.
- Added `read_parallel_scan_selected_result_slot_snapshot(...)`, which:
  - reads the coordinator snapshot
  - follows the coordinator-selected slot index when present
  - validates that the named slot is still live
  - returns the coordinator snapshot plus the selected slot snapshot
- Added coverage for:
  - fast-path read returning `None` when no selected staged result exists
  - fast-path read returning the current coordinator-selected staged result
  - fast-path read following the refreshed coordinator snapshot after the
    selected slot is cleared
- Updated Task 18 notes so the live staged status reflects the direct
  coordinator fast-path seam.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no shared top-K heap mutation path yet
- no worker-local traversal scratch in DSM yet
- no planner-visible parallel execution yet

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `bash scripts/run_pg18_preload_pgstat_test.sh`

Review focus:
- Whether the direct coordinator fast-path reader is the right seam ahead of
  real shared top-K drain/merge work
- Whether validating the named staged slot rather than blindly trusting the
  coordinator snapshot is the right safety contract
- Whether this is enough direct coordinator access for the next narrow merge
  slice without overcommitting the eventual shared top-K API
