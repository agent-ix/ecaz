# Review Request: Parallel Scan Heap-Root Pop Drain

Current head: `463d4f9`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The prior Task 18 slice added a shared coordinator-owned min-heap over the
  one-live-result-per-worker staged frontier.
- But coordinator staged-result take still cleared the selected slot and then
  rebuilt the entire heap to refresh the next selected result.
- We needed the next seam to drain that staged frontier more directly before
  the real lock-guarded shared top-K admission path lands.

What changed:
- Lifted the shared heap helper surface in `src/am/common/parallel.rs` so heap
  entry ordering, swap, sift-up, sift-down, rebuild, fast-path refresh, and
  root pop are explicit helpers instead of nested one-off logic.
- `take_parallel_scan_selected_result_slot_snapshot(...)` now:
  - clears the selected staged result slot without forcing a full refresh
  - pops the shared heap root in place
  - refreshes the coordinator fast-path snapshot from the next heap root when
    one exists
  - falls back to a full refresh only if the selected slot and heap root have
    drifted out of sync
- `clear_parallel_scan_result_slot_with_attachment(...)` now accepts a
  `refresh_selection_snapshot` flag so callers that need a full refresh can
  still request it, while coordinator take can use the narrower in-place drain
  path.
- Updated Task 18 notes to say staged coordinator take now pops the shared heap
  root instead of rebuilding the heap after every consume.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no lock-guarded shared top-K push/pop admission path yet
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
- Whether the new heap helper split keeps the coordinator drain path legible
  without over-committing to the final shared top-K mutation API
- Whether the clear-without-refresh plus root-pop contract is the right narrow
  seam for the next shared admission/merge packet
- Whether the fallback-to-full-refresh cases are sufficient to keep the staged
  heap and selected-result snapshot coherent
