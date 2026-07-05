# Review Request: Concurrent DSM Graph Readback

Current head: `48eda22`

Scope:
- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- ADR-048 keeps page staging unchanged after concurrent DSM graph assembly.
- Packet 638 added DSM image initialization.
- Packet 639 added participant node-range planning and entry-node bootstrap
  state.
- The leader now needs a tested bridge from a completed DSM graph back into the
  existing `HnswBuildNode` page-staging shape.

Change:
- Exposed `flatten_native_neighbor_slots` within the `ec_hnsw` module so DSM
  readback can reuse the same score-neighbor flattening contract as the serial
  native builder.
- Added `concurrent_dsm_graph_to_build_nodes`, which:
  - requires every DSM node to be in `READY` state
  - maps invalid neighbor-slot sentinels to `None`
  - rejects out-of-range neighbor indexes
  - reconstructs `build::HnswBuildNode` with level, neighbor slots, and
    flattened score-neighbor IDs
- Added tests for successful readback, uninserted-node rejection, and
  out-of-range neighbor rejection.

Validation:
- `cargo test concurrent_dsm_graph_readback`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether this is the right post-assembly bridge before the DSM graph is wired
  into a real graph phase.
- Whether requiring all nodes to be `READY` before page staging is strict
  enough.
- Whether reusing `flatten_native_neighbor_slots` is preferable to duplicating
  score-neighbor flattening in `build_parallel.rs`.
