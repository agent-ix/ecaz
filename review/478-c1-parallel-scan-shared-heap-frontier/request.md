# Review Request: Parallel Scan Shared Heap Frontier

Current head: `ad08a37`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The prior Task 18 slices could publish one staged current result per worker,
  select the best staged slot, and drain that staged selection.
- But the shared descriptor still did not carry a real coordinator-owned heap
  surface. Selection rescanned the staged result slots directly each refresh,
  which left the next top-K packet without an explicit shared frontier layout
  to build on.
- We needed the first real shared min-heap over the one-live-result-per-worker
  frontier while keeping the descriptor query-independent and `amcanparallel`
  off.

What changed:
- Added shared heap layout metadata to `EcParallelScanState` and
  `ParallelScanAttachment` in `src/am/common/parallel.rs`.
- Added `EcParallelCoordinatorHeapState`,
  `EcParallelCoordinatorHeapSnapshot`, heap sizing helpers, and a shared heap
  entry array in the AM-private descriptor.
- `reset_parallel_scan_layout(...)` now initializes the heap header and clears
  the heap entry array back to the invalid sentinel.
- `refresh_coordinator_selection_snapshot(...)` now:
  - reaps dead staged result slots first
  - rebuilds a coordinator-owned min-heap over the currently live staged
    worker results
  - records heap liveness/generation in shared state
  - refreshes the selected-result fast path from the heap root
- `select_best_parallel_scan_coordinator_result_slot_with_attachment(...)`
  now reads the heap root instead of rescanning all staged result slots.
- Added direct tests for:
  - descriptor sizing including the shared heap header/entries
  - empty heap initialization
  - heap snapshot/root state after staged publishes
- Updated Task 18 notes to say the descriptor now carries a shared min-heap
  over the staged one-result-per-worker frontier.

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
- Whether the one-entry-per-worker shared heap is the right explicit frontier
  surface for the next real coordinator merge/admission slice
- Whether rebuild-on-refresh keeps the staging contract narrow enough before
  the lock-guarded mutation path lands
- Whether the descriptor/version/layout changes stay coherent across PG17 and
  PG18 callback surfaces
