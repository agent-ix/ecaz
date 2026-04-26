# Review Request: Concurrent DSM Layout Reattach

Current head: `12d11b9`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- Packet `643` proved that a completed participant-produced DSM graph can feed
  the existing current-format page-staging path.
- The next wiring blocker is worker reattachment: future worker processes will
  receive a DSM pointer through `ParallelContext`, not the leader-local
  `EcHnswConcurrentDsmPreassemblyPlan`.
- Workers therefore need to reconstruct the graph layout from the initialized
  DSM header before deriving node, slot, and code-section pointers.

Change:
- Refactored `EcHnswConcurrentDsmGraphLayout::for_levels` through a shared
  header-value constructor.
- Added attach-time layout reconstruction from an initialized DSM graph header.
- Added `concurrent_dsm_graph_layout_from_image(base)` so future workers can
  rebuild the layout from only a DSM base pointer.
- Added validation that a non-empty DSM graph header must include a valid entry
  node.
- Added tests proving:
  - an initialized DSM image reattaches to the exact leader-computed layout
  - reattached parts point to the same header, node array, neighbor slots, and
    code corpus
  - malformed non-empty headers without an entry node are rejected

Validation:
- `cargo test concurrent_dsm_graph_layout -- --nocapture`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether reconstructing offsets from the DSM header is the right worker-attach
  contract.
- Whether the header contains enough durable metadata for worker-side graph
  insertion, or whether additional fields should be added before `shm_toc`
  wiring.
- Whether rejecting non-empty graphs without an entry node is the right
  invariant at attach time.
