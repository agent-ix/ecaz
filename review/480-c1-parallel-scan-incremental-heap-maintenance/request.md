# Review Request: Parallel Scan Incremental Heap Maintenance

Current head: `f3da933`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The prior Task 18 slices gave the coordinator a staged shared min-heap plus
  root-pop drain, but worker publish and clear still refreshed selection by
  reaping and rebuilding the entire staged heap.
- That kept the staged frontier correct, but it over-modeled the eventual
  shared top-K path and made every per-slot mutation pay an O(n) rebuild cost
  even though only one staged result slot had changed.
- The next Task 18 packets need a narrower incremental contract that preserves
  staged-heap ordering across single-slot publish/clear/take operations.

What changed:
- Added reverse slot-to-heap membership in `EcParallelCoordinatorResultSlot`
  and bumped the shared parallel descriptor version to match the DSM layout
  change.
- Split the heap helpers so staged result slots can now:
  - store and load their current heap membership
  - detach themselves from the shared heap in place
  - upsert and reheapify themselves in place after publish
- Coordinator fast-path refresh now lazily reaps stale dead roots from the
  staged heap instead of requiring a full heap rebuild for ordinary publish,
  clear, and staged take paths.
- `publish_parallel_scan_coordinator_result_slot_runtime_snapshot(...)` now
  upserts one slot into the shared staged heap and refreshes the fast path.
- `clear_parallel_scan_result_slot_with_attachment(...)` and staged
  coordinator take now detach the affected slot from the heap in place before
  clearing the slot state.
- Added focused regression coverage for republishing an already-staged slot
  with a lower score and verifying that the coordinator heap and fast path move
  to that slot without a full rebuild.
- Updated Task 18 notes to state that staged-heap maintenance is now
  incremental.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no lock-guarded shared top-K admission path yet
- no worker-owned traversal scratch in DSM yet
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
- Whether reverse slot-to-heap membership is the right staged boundary before
  the eventual lock-guarded shared top-K admission API lands
- Whether lazy dead-root reap is the right fallback for stale claims, rather
  than keeping full-heap rebuild on the ordinary publish/clear path
- Whether the new detach/update helpers keep the heap and coordinator fast path
  coherent under slot republish, clear, and staged take
