# Review Request: Concurrent DSM Graph Attachment

Current head: `5d671ab`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- Packet `644` added attach-time reconstruction of DSM graph section offsets
  from the initialized graph header.
- Packet `645` added insert/scoring config fields to the graph header so future
  workers can reconstruct `EcHnswConcurrentDsmInsertConfig` from a DSM base
  pointer.
- The next worker-wiring step needs a single attach result that groups the raw
  DSM parts, reconstructed layout, and optional insert config.

Change:
- Added `EcHnswConcurrentDsmGraphAttachment` as the worker/leader attach
  boundary for a concurrent DSM graph image.
- Added `attach_concurrent_dsm_graph_image(base)` to reconstruct:
  - `EcHnswConcurrentDsmGraphLayout`
  - `EcHnswConcurrentDsmInsertConfig`
  - `EcHnswConcurrentDsmGraphParts`
- Added `require_insert_config()` so non-empty graph worker/completion paths can
  fail at the attach boundary if insert metadata is missing.
- Added `current_format_flush_output_from_concurrent_dsm_graph(...)` so the
  leader can convert a completed attached DSM graph directly into current-format
  page-staging output.
- Updated DSM graph tests to consume the attachment object for layout reattach,
  empty-header attach, and single-participant current-format staging.

Validation:
- `cargo test concurrent_dsm_graph -- --nocapture`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether `EcHnswConcurrentDsmGraphAttachment` is the right boundary for the
  upcoming `ParallelContext` worker callback.
- Whether `require_insert_config()` should be the failure mode for graph
  insertion/completion paths, while empty builds keep optional config.
- Whether the leader-side completion helper is the right staging boundary before
  the final parallel build coordinator wiring.
