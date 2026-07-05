# Review Request: Parallel Concurrent DSM Graph Workers

Current head: `27cdf09`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`
- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/options.rs`
- `src/lib.rs`

Context:
- Packets `643` through `646` established the concurrent DSM graph layout,
  insertion helpers, attach contract, and current-format page-staging boundary.
- The remaining blocker was an executable PostgreSQL worker path: workers had
  not yet attached the DSM graph from `ParallelContext` or inserted graph
  partitions in a real PG18 build.

Change:
- Added opt-in GUC `ec_hnsw.enable_parallel_build_concurrent_dsm`.
- Added a second parallel worker entrypoint:
  `ec_hnsw_parallel_graph_build_main`.
- After the existing parallel heap-ingest phase, opt-in eligible TurboQuant
  builds now launch a graph-assembly `ParallelContext`.
- The graph context:
  - allocates and initializes the concurrent DSM graph image
  - registers DSM graph node LWLocks under a build-local tranche
  - exposes the graph image through `shm_toc`
  - has workers attach via `attach_concurrent_dsm_graph_image`
  - has workers insert their participant partitions directly into the DSM graph
  - has the leader cover missing-worker partitions and the leader partition
  - converts the completed DSM graph to current-format page-staging output
- Added a pg_test opt-in smoke test that sets the GUC, creates a parallel
  ec_hnsw index, asserts graph workers launched, and verifies all heap TIDs are
  present with a valid entry point.
- Added a debug test helper for graph-worker launch count.

Validation:
- `cargo test build_parallel -- --nocapture`
- `cargo pgrx test pg18 test_pg18_parallel_index_build_concurrent_dsm_graph_opt_in`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo test`
- `cargo pgrx test pg18`
- `git diff --check`

Review focus:
- Whether the transitional two-phase design is acceptable: existing shm_mq heap
  ingestion first, then a dedicated graph-assembly `ParallelContext`.
- Whether leader coverage of missing-worker partitions is the right fallback
  when fewer graph workers launch than requested.
- Whether the opt-in GUC is the right safety gate before recall and build-time
  measurement packets.
- Whether the graph worker instrumentation/debug surface is sufficient for the
  next recall/speed validation slice.
