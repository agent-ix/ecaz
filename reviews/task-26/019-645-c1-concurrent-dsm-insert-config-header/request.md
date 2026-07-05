# Review Request: Concurrent DSM Insert Config Header

Current head: `23ea2b1`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- Packet `644` added attach-time reconstruction of DSM graph section offsets
  from the graph header.
- Future graph workers also need scoring and insertion metadata after attaching
  to the DSM image. They cannot read the leader-local `BuildState`.
- The required metadata is the existing `EcHnswConcurrentDsmInsertConfig`:
  dimensions, bits, seed, `m`, and `ef_construction`.

Change:
- Extended the concurrent DSM graph header with insert-config fields:
  - dimensions
  - bits
  - seed
  - `m`
  - `ef_construction`
- Added optional `insert_config` to `EcHnswConcurrentDsmPreassemblyPlan`.
  Empty builds carry no config; non-empty builds copy it from `BuildState`.
- `initialize_concurrent_dsm_graph_image` now writes the insert config into the
  DSM header alongside graph layout metadata.
- Added `concurrent_dsm_insert_config_from_image(base)` so future worker
  attach code can reconstruct insertion/scoring config from only the DSM base
  pointer.
- Added validation that non-empty attached graphs reject missing dimensions,
  bits, `m`, or `ef_construction`.

Validation:
- `cargo test concurrent_dsm -- --nocapture`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether storing insert/scoring metadata in the graph header is the right
  worker-attach contract.
- Whether empty graphs should keep returning `None` for insert config.
- Whether the validation checks are sufficient before the worker callback uses
  this header data directly.
