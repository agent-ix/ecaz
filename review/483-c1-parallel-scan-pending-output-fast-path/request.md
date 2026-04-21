# Review Request: Parallel Scan Pending-Output Fast Path

Current head: `051804f`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The prior slice let the coordinator drain one pending heap TID at a time from
  a staged worker-result slot, but the shared coordinator snapshot still cached
  only the selected slot index and score.
- That meant readers still had to reconstruct the next pending output from the
  selected slot, and after the first pending heap TID was consumed the cached
  coordinator fast path could lag behind the slot's new pending index.
- The next merge/admission work wants the coordinator snapshot to name the next
  global pending output directly, not just the owning slot.

What changed:
- Extended `EcParallelCoordinatorState` and `EcParallelCoordinatorSnapshot` so
  the coordinator now caches the selected pending output itself:
  - heap TID
  - score
  - optional approx score
  - optional comparison score
  - optional approx rank
- Added `EC_PARALLEL_COORDINATOR_SELECTED_PENDING_OUTPUT_VALID` and refreshed
  the coordinator fast path so it now updates both:
  - the selected staged worker-result slot
  - the selected pending output derived from that slot
- Added `read_parallel_scan_selected_pending_output_snapshot(...)` so later
  merge work can read the next global pending output directly without taking it.
- Fixed a real stale-fast-path bug found by the new tests: when
  `take_parallel_scan_selected_pending_output_snapshot(...)` advances within a
  live slot, it now refreshes the coordinator fast path so the cached pending
  output moves to the next heap TID instead of staying on the one just emitted.
- Added focused coverage for:
  - direct pending-output read through the coordinator fast path
  - advancing the cached pending output after consuming the first heap TID in a
    multi-output staged slot
- Updated Task 18 notes to say the coordinator now caches the selected pending
  output directly.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no shared top-K admission heap yet
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
- Whether caching the selected pending output in coordinator state is the right
  next seam before introducing the real shared top-K admission heap
- Whether the coordinator fast-path refresh now owns the right contract after a
  within-slot pending-output advance
- Whether the new direct pending-output read helper is narrow enough for the
  later merge/admission slices without overcommitting the final API
