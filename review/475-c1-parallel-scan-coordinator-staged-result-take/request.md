# Review Request: Parallel Scan Coordinator Staged-Result Take

Current head: `20b09c5`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The prior Task 18 slice could read the coordinator-selected staged result
  directly, but there was still no helper that could consume that staged result
  and advance the coordinator fast path to the next best staged result.
- That left the next coordinator merge/drain slice without a narrow
  consume-and-advance seam, even though staged publication, selection, and
  direct reads were already in place.
- We needed a staged-result take helper that remains strictly within the staged
  slot contract and still defers the real shared top-K heap.

What changed:
- Added `take_parallel_scan_selected_result_slot_snapshot(...)`, which:
  - reads the current coordinator-selected staged result
  - validates that the named slot is still live
  - clears that staged slot
  - refreshes the coordinator snapshot to the next best staged result, if any
  - returns the consumed staged result snapshot
- Added focused coverage for:
  - `None` when no staged result is selected
  - taking the only staged result clears the coordinator fast path
  - taking the current selected staged result refreshes the coordinator fast
    path to the next best staged result
- Updated Task 18 notes so the live staged status reflects that coordinator
  consumers can now drain one staged result at a time without enabling the real
  shared top-K heap.

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
- Whether a staged-result consume helper is the right next seam before the real
  shared top-K heap lands
- Whether consume-and-refresh via slot clear is the right lifetime contract for
  later coordinator merge work
- Whether this staged one-at-a-time drain API is sufficient for the next narrow
  coordinator slice without overcommitting the final top-K interface
