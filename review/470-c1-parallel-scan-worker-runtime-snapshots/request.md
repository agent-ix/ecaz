# Review Request: Parallel Scan Worker Runtime Snapshots

Current head: `0a8da4e`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The prior Task 18 slice let `TqScanOpaque` claim and release a shared worker
  slot, but the slot still carried only ownership metadata.
- That left no stable shared contract for later coordinator logic to observe
  worker-local scan progress without reaching back into private `TqScanOpaque`
  state.
- The next Task 18 slices need a shared carrier for worker phase and staged
  traversal counts before the coordinator heap or planner-visible parallel path
  can land cleanly.

What changed:
- Expanded `EcParallelWorkerSlot` to carry a staged runtime snapshot for:
  - execution phase
  - scan dimensions
  - bootstrap frontier limit
  - visible-frontier length
  - scheduler frontier length
  - visited count
  - emitted-result count
  - active-result pending count
  - whether the active result still has a current row
- Added shared helpers to:
  - publish a runtime snapshot into a claimed slot for the active rescan epoch
  - read back the full slot snapshot for tests and later coordinator work
  - reset slot runtime back to an idle zero snapshot on release or rescan
- `ec_hnsw` now publishes slot snapshots at scan lifecycle boundaries:
  - bind
  - reset
  - end-of-rescan setup
  - linear fallback transition
  - exhaustion
- Added coverage for:
  - runtime snapshot round-trip
  - stale-epoch publish rejection
  - scan-side runtime snapshot mirroring

Still intentionally deferred:
- `amcanparallel` remains `false`
- no shared top-K heap or coordinator merge path yet
- no worker-slot-owned beam/frontier/visited memory yet
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
- Whether the worker-slot runtime snapshot fields are the right staged boundary
  for later coordinator and EXPLAIN rollup work
- Whether publishing only at scan lifecycle boundaries is the right tradeoff for
  this stage, rather than wiring the shared slot into hot-path per-event updates
- Whether release/rescan reset semantics leave the slot in the right clean state
  for the next claimant
