# Review Request: Parallel Index Build Coordinator Scaffold

Current head: `1812f1e`

Scope:
- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/build_parallel.rs`
- `src/am/ec_hnsw/mod.rs`
- `src/quant/rabitq.rs`

Problem:
- Task 18 parallel scan work is shelved because the current hidden-result
  handoff strategy is not a credible path to scan speedup.
- Parallel index build has a more direct useful first phase: split heap scan
  and tuple encoding across workers, then keep HNSW graph assembly on the
  leader until the serial graph-write contract is isolated.
- The existing scan coordinator in `src/am/common/parallel.rs` is not the
  right build coordinator. It is shaped around scan descriptor attachment,
  rescan epochs, worker slot ownership, and traversal/runtime snapshots rather
  than around a build DSM, parallel heap ingestion, encoded tuple transport,
  and leader-side graph assembly.

What changed:
- Added `src/am/ec_hnsw/build_parallel.rs` as a dedicated parallel-build
  planning boundary.
- The new plan explicitly models:
  - serial leader-local builds for normal requests
  - dedicated parallel build coordination when workers are requested
  - parallel table scan ingestion
  - shared encoded tuple transport
  - serial leader graph assembly
- Added a versioned shared-header scaffold for future DSM wiring, with atomic
  counters for worker heap/index tuple accounting.
- Wired `ec_hnsw_ambuild` through the new plan while keeping the normal serial
  build path unchanged.
- Kept the executable parallel path gated: unexpected parallel build requests
  currently error with a clear message instead of falling through to a partial
  implementation.
- Kept `amcanbuildparallel = false` unchanged. This checkpoint does not ask
  PostgreSQL to plan parallel index builds yet.
- Fixed two unrelated `RaBitQ` range loops that blocked the required PG18
  clippy lane under the current toolchain.

Validation:
- Passed:
  - `cargo test`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `git diff --check`

Review focus:
- Whether the dedicated `build_parallel` coordinator boundary is the right
  shape for build work, instead of trying to reuse the scan coordinator.
- Whether the staged plan is conservative enough: parallel heap ingestion and
  tuple encoding first, serial leader graph assembly initially.
- Whether keeping `amcanbuildparallel = false` until the DSM worker entrypoint
  and tuple transport are executable is the right gating point.
- Whether the shared-header scaffold has the right early fields before adding
  `CreateParallelContext`, `table_beginscan_parallel`, and worker callbacks.
