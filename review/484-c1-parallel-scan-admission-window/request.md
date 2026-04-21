# Review Request: Parallel Scan Admission Window

Current head: `9590e43`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The coordinator could already stage worker-owned pending outputs and drain
  them one at a time, but there was still no shared admission surface for the
  coordinator to retain accepted outputs independently of the worker-result
  heap.
- The next Task 18 slices need a narrow shared contract for:
  - admitting the currently selected pending output
  - keeping an ordered admitted window in DSM
  - tracking admission count, generation, and current worst score
- This slice also needed to keep the existing coordinator fast path correct as
  the new admission metadata started sharing the coordinator flag word.

What changed:
- Added a staged coordinator-owned admitted-result window to the shared parallel
  DSM layout:
  - `EcParallelCoordinatorAdmittedResult`
  - admitted-result size/capacity accounting in `EcParallelScanState`
  - admitted-result attachment pointers and validation
- Extended coordinator state and snapshots with admission metadata:
  - `admitted_result_count`
  - `admitted_result_generation`
  - `admitted_worst_score`
- Added admission helpers and read surfaces:
  - `read_parallel_scan_admission_snapshot(...)`
  - `read_parallel_scan_admitted_result_snapshot(...)`
  - `admit_parallel_scan_selected_pending_output(...)`
- Admission now:
  - inserts new pending outputs into score-ordered admitted storage
  - rejects duplicate heap TIDs
  - rejects worse candidates once the admitted window is full
  - replaces the admitted tail when a better candidate arrives
- Fixed the flag-seam bug this surfaced: selected-result fast-path refresh now
  preserves the admitted-window validity bit instead of clobbering it.
- Updated the scan-side bind test scratch storage to match the larger parallel
  descriptor size after the admission window landed.
- Added focused coverage for:
  - first admission into an empty window
  - duplicate rejection
  - ordered insertion
  - full-window replacement and worse-candidate rejection

Still intentionally deferred:
- `amcanparallel` remains `false`
- no shared top-K mutation heap yet beyond this ordered admitted window
- no planner-visible parallel execution yet
- no worker-owned traversal scratch in DSM yet

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `bash scripts/run_pg18_preload_pgstat_test.sh`

Review focus:
- Whether the admitted-result window is the right narrow seam before the later
  shared top-K / planner-enablement slices
- Whether score ordering, duplicate rejection, and full-window replacement are
  the right staged admission semantics
- Whether preserving the admitted-window validity bit in the coordinator fast
  path is the right ownership boundary for shared coordinator flags
