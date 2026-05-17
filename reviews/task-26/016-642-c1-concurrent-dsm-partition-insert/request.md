# Review Request: Concurrent DSM Partition Insert

Current head: `d184851`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- Packet `641` added the worker-callable single-node insertion path for the
  concurrent DSM graph.
- Workers and leader participants need a narrow loop over their assigned
  half-open node ranges before that insertion core can be wired into the
  parallel build callback.

Changes:
- Added `insert_concurrent_dsm_graph_partition`, which validates a planned
  `EcHnswConcurrentDsmNodePartition` against the DSM graph layout and inserts
  each node in `start_node_idx..end_node_idx`.
- Preserved idempotent behavior from the single-node helper: preinitialized
  entry nodes and already-ready nodes are skipped, not counted as inserted.
- Added overflow-checked inserted-node accounting for worker result summaries.
- Added a test proving two partitions cover a four-node graph once, skip the
  preinitialized entry node, and do not reinsert an already-ready partition.

Validation:
- `cargo test concurrent_dsm_graph_partition_insert -- --nocapture`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether the partition helper is the right boundary for the upcoming
  leader/worker callback wiring.
- Whether returning only the inserted count is sufficient for the first worker
  result surface, or whether reviewers want skipped/error counters before the
  callback is connected.
- Whether the idempotent READY skip is acceptable for retry-safe callback
  behavior, given that out-of-order concurrent insertion is expected.
