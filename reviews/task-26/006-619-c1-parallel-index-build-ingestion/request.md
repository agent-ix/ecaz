# Review Request: Parallel Index Build Ingestion

Current head: `4e7010b`

Scope:
- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/build_parallel.rs`
- `src/am/ec_hnsw/mod.rs`
- `src/am/ec_hnsw/routine.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Problem:
- Task 18 parallel scan work is shelved. The scan coordinator was not producing
  a credible speedup path and is the wrong abstraction for index builds.
- Parallel index build has a cleaner first useful slice: let PostgreSQL workers
  split heap ingestion and tuple encoding, then keep HNSW graph/page assembly on
  the leader until the graph-write contract is isolated.
- The existing scan coordinator in `src/am/common/parallel.rs` is intentionally
  not reused. It coordinates scan descriptor attachment, worker slots, rescan
  epochs, and traversal/runtime snapshots. Build needs a DSM/table-scan/message
  queue coordinator with leader-side merge semantics.

What changed:
- Enabled `amcanbuildparallel` for `ec_hnsw`.
- Replaced the scaffolded parallel-build error path with an executable PG18
  build coordinator in `src/am/ec_hnsw/build_parallel.rs`.
- The leader now creates a PostgreSQL `ParallelContext`, initializes a
  `ParallelTableScanDesc` in DSM, launches workers, and allocates one `shm_mq`
  per worker.
- Workers run `table_beginscan_parallel` plus `table_index_build_scan`, encode
  each heap tuple with the existing build tuple path, and send tuple messages to
  the leader.
- The leader drains all worker queues, sorts received build tuples by heap TID,
  pushes them into the existing `BuildState`, and keeps the existing serial HNSW
  graph/page writer unchanged.
- Added a PG18 smoke test that requests parallel maintenance workers, builds an
  `ec_hnsw` index, verifies at least one worker launched, and verifies the
  resulting index contains all 128 heap TIDs with a valid entry point.
- Added test-only debug plumbing for the last launched parallel-build worker
  count.

Current limitations:
- The leader does not participate in the heap scan in this slice. It stays
  dedicated to draining worker message queues to avoid queue-fill deadlocks.
- `build_source_column` indexes currently fall back to the serial build path.
- Graph assembly and page writes remain serial on the leader.
- This packet makes no performance claim; it proves the PG18 worker/DSM/MQ path
  is functional and leaves measurement for a later packet.

Validation:
- Passed:
  - `cargo test build_parallel --lib`
  - `cargo pgrx test pg18 test_pg18_parallel_index_build_uses_workers`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `cargo test`
  - `cargo pgrx test pg18`
  - `git diff --check`

Review focus:
- Whether the dedicated build coordinator remains the right boundary now that
  the executable path exists.
- Whether keeping graph assembly serial while parallelizing heap ingestion and
  encoding is the right first checkpoint.
- Whether leader-drains-only is acceptable for this slice, or if leader
  participation should be prioritized before measurement.
- Whether the tuple message format and per-worker `shm_mq` topology are
  acceptable as an initial transport before adding shared sorting or a more
  compact tuple stream.
