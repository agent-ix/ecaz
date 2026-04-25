# Review Request: Concurrent DSM Insertion Ranges

Current head: `41f0c4f`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- Packet 638 added the DSM image initializer for the concurrent graph surface.
- The insertion phase still needs a deterministic participant-to-node contract
  before worker wiring starts.
- The fixed entry node also needs to be queryable from the beginning of the
  insertion phase.

Change:
- Added `EcHnswConcurrentDsmNodePartition`, a half-open node range assigned to
  one participant.
- Added `concurrent_dsm_node_partitions`, which splits node indexes evenly
  across participants and assigns any remainder to earlier participants.
- Allows empty tail ranges when worker count exceeds node count.
- Rejects zero participants.
- Updated DSM image initialization so the fixed entry node starts in `READY`
  state and all other nodes start `UNINSERTED`.

Validation:
- `cargo test concurrent_dsm_node_partitions`
- `cargo test concurrent_dsm_graph_image`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether static half-open node ranges are the right first worker partition
  contract for the concurrent insertion spike.
- Whether marking the fixed entry node ready at DSM initialization is the right
  bootstrap rule before other participants begin searching.
- Whether this should stay as a pure planner helper or move into a future
  graph-phase shared header when worker wiring lands.
