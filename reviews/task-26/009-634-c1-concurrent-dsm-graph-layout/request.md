# Review Request: Concurrent DSM Graph Layout

Current head: `e800e6d`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`
- `src/am/ec_hnsw/build.rs`

Context:
- ADR-048 now targets concurrent HNSW graph insertion into DSM with one LWLock
  per node.
- Packet 633 added explicit native build-level precomputation.
- PostgreSQL workers are separate processes, so graph workers cannot score
  candidates from the leader's Rust `BuildState`. The DSM surface therefore
  needs the shared graph plus compact encoded code bytes. This still avoids
  pgvector's raw-float DSM pressure.

Change:
- Added C-compatible DSM graph header and node structs.
- Modeled each DSM node with:
  - one `LWLock`
  - level
  - neighbor-slot offset/count into a flat `u32` slot array
  - insertion state placeholder for the future concurrent insertion protocol
- Added `EcHnswConcurrentDsmGraphLayout` to size:
  - header
  - node array
  - flat neighbor-slot array
  - compact code corpus bytes
- Added layout tests for non-empty and empty level plans.

Validation:
- `cargo test concurrent_dsm_graph_layout`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether the DSM memory model is the right next surface for the concurrent
  insertion path.
- Whether the compact code corpus belongs in this DSM layout or should be
  separated into a distinct shared corpus object.
- Whether `insert_state` is the right placeholder for avoiding reads from
  uninserted nodes once the fixed entry point and concurrent insertion protocol
  are wired.
