# Review Request: Native Build Sparse Query Score Cache Measurement

Current head: `373a363`

Scope:
- `src/am/ec_hnsw/build.rs`
- `review/624-c1-native-build-sparse-query-score-cache-measurement/artifacts/manifest.md`
- `review/624-c1-native-build-sparse-query-score-cache-measurement/artifacts/pg18_sparse_query_score_cache_timing.sql`
- `review/624-c1-native-build-sparse-query-score-cache-measurement/artifacts/pg18_sparse_query_score_cache_timing.log`

Question:
- Does replacing the native HNSW graph builder's per-node dense query score
  cache allocation with a reusable generation-stamped cache reduce the graph
  construction phase identified in packet 622?

Code Change:
- `NativeBuildQueryScorer` no longer allocates `vec![None; heap_tuples.len()]`
  for every inserted node.
- `build_native_hnsw_graph` now owns one `NativeBuildQueryScoreCache` for the
  whole build. Each inserted node advances a generation counter, so score
  lookups remain dense O(1) without clearing the full cache each round.

Result:
- Fixture: 10,000 rows, 64 dimensions, default `turboquant`, no
  `build_source_column`, `m = 6`, `ef_construction = 40`.
- Serial create-index times:
  - round 1: 9086.669 ms
  - round 2: 8559.489 ms
- Parallel create-index times:
  - round 1: 8882.599 ms
  - round 2: 8514.682 ms
- Average serial: 8823.079 ms, versus 9013.102 ms in packet 622.
- Average parallel: 8698.641 ms, versus 8694.528 ms in packet 622.
- All measured index sizes were identical at 2,334,720 bytes.

Phase Breakdown:
- Average serial graph construction: 8197.469 ms, versus 8384.270 ms in
  packet 622.
- Average parallel graph construction: 8220.860 ms, versus 8214.275 ms in
  packet 622.
- Average serial heap ingest: 541.829 ms.
- Average parallel heap ingest total: 396.394 ms.
- Average parallel queue drain/finish: 133.722 ms.
- Average parallel sort plus `BuildState::push`: 259.998 ms.

Interpretation:
- The change removes a real per-node O(N) cache initialization in the graph
  builder and modestly improves the serial graph phase on this committed-head
  run.
- The parallel build wall time is effectively neutral on this 10k fixture:
  heap ingestion still benefits from workers, but serial graph construction
  remains dominant.
- This does not change the broader direction from packet 622. Parallel scan
  transport is not the limiting factor; further wins require reducing or
  parallelizing graph construction itself.

Validation:
- Passed:
  - `cargo test hnsw_graph_build --lib`
  - `cargo pgrx test pg18 test_pg18_parallel_index_build_uses_workers`
  - `cargo test`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `cargo pgrx test pg18`
  - `git diff --check`
- Raw measurement log is stored packet-locally at
  `artifacts/pg18_sparse_query_score_cache_timing.log`.
- SQL fixture is stored packet-locally at
  `artifacts/pg18_sparse_query_score_cache_timing.sql`.
- Artifact metadata and key lines are recorded in `artifacts/manifest.md`.
- The run used `ecaz dev sql --pg 18 --log-output`, not shell redirection or
  `script`.

Review focus:
- Whether the generation-stamped score cache is acceptable as a small graph
  construction cleanup.
- Whether the neutral parallel result is enough evidence to keep prioritizing
  graph construction over parallel heap-ingest tuning.
