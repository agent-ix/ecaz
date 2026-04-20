# Review Request: Parallel Scan Coordinator Result Slots

Current head: `575210f`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The prior Task 18 slices established the shared descriptor, worker-slot
  claiming, and worker runtime snapshots, but there was still no shared
  contract for the coordinator side of result publication.
- That left the next top-K merge slice without a stable DSM seam for current
  result identity/score state, and it risked forcing another descriptor rewrite
  once the real coordinator heap lands.
- We needed a staged coordinator-owned result carrier that stays aligned with
  the existing rescan-epoch and worker-slot ownership rules.

What changed:
- Expanded `EcParallelScanState` and `ParallelScanAttachment` to reserve one
  coordinator-owned staged result slot per worker slot in the shared DSM
  descriptor.
- Added `EcParallelCoordinatorResultSlot` plus helper snapshots for:
  - published-result slot flags
  - element and heap TIDs
  - score / approx score / comparison score
  - approx-rank base
  - pending-result count and cursor
- Added shared helpers to:
  - publish a coordinator result slot for a claimed worker in the active rescan
    epoch
  - clear that slot on demand
  - read back coordinator and result-slot snapshots for tests and later
    coordinator work
  - clear the staged result slot automatically when worker-slot release or
    rescan resets the descriptor
- `ec_hnsw` now publishes the active current-result state alongside the worker
  runtime snapshot at the existing scan lifecycle boundaries.
- Added coverage for:
  - coordinator result-slot publish round-trip
  - clear/reset semantics
  - release-side staged-result cleanup
  - scan-side mirroring of current-result state into the shared slot
- Updated Task 18 notes so the staged status matches the live coordinator seam.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no shared top-K heap ordering or merge path yet
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
- Whether one staged coordinator-owned current-result slot per worker is the
  right interim seam before the real shared top-K heap lands
- Whether tying coordinator result-slot mutation to worker-slot ownership and
  rescan epochs is the right lifetime contract
- Whether the published result fields are sufficient for the next merge/EXPLAIN
  rollup slices without overcommitting the final coordinator layout
