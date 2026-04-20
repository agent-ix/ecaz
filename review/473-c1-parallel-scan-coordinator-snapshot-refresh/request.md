# Review Request: Parallel Scan Coordinator Snapshot Refresh

Current head: `663d74a`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The prior Task 18 slice could scan the staged coordinator result slots and
  choose the current best one, but the coordinator header itself still did not
  carry a direct snapshot of that chosen result.
- That meant later coordinator work would still have to rescan staged slots to
  answer simple "what is the current best staged result?" questions.
- We needed the coordinator header to publish that snapshot directly while
  still keeping the shared path read-only and leaving top-K mutation deferred.

What changed:
- Extended `EcParallelCoordinatorState` with a coordinator-owned snapshot of:
  - selected staged result slot index
  - selected staged result score
- Extended `EcParallelCoordinatorSnapshot` to expose that state as:
  - `selected_result_slot_index`
  - `selected_result_score`
- Factored staged-slot selection into a shared attachment-local helper so it
  can be reused both by the explicit selection API and by coordinator refresh.
- Coordinator snapshot refresh now runs on:
  - staged-result publish
  - staged-result clear
  - worker-slot release via staged-result clear
  - descriptor reset/rescan through layout reinitialization
- Added focused coverage for:
  - selected-result state appearing in the coordinator snapshot after publish
  - selected-result state following the best slot across multiple publishes
  - selected-result refresh after clearing the currently selected slot
  - reset/clear/release returning the coordinator snapshot to `None`
- Updated Task 18 notes so the staged coordinator snapshot seam is reflected in
  the live task text.

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
- Whether the coordinator-owned selected-result snapshot is the right next seam
  ahead of shared top-K heap mutation
- Whether refreshing that snapshot on publish/clear/reset is the right
  lifetime contract for later coordinator merge work
- Whether carrying just slot index plus score is sufficient at this stage
  without overcommitting the eventual shared top-K layout
