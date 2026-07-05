# Review Request: Concurrent DSM Graph Page Staging

Current head: `64f5ddb`

Scope:
- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- Feedback packet `632` revised ADR-048 toward concurrent HNSW insertion into
  a DSM-resident graph with per-node locks.
- Packets `638` through `642` added the DSM graph image, readback, node
  insertion, and participant partition helpers.
- The next narrow proof is that a participant-produced DSM graph can be handed
  to the existing current-format page-staging path without changing the page
  writer contract.

Change:
- Split `current_format_flush_output` so serial native graph construction and
  current-format page staging are separate steps.
- Added `current_format_flush_output_from_graph_nodes` for tests and upcoming
  DSM graph completion wiring. It validates that graph node count matches the
  build tuple count before staging pages.
- Added `insert_concurrent_dsm_graph_participant`, which maps a participant
  index/count pair to the existing deterministic DSM node partition helper and
  inserts that partition.
- Added a leader-only participant test that inserts into the DSM graph, reads
  the DSM graph back into `HnswBuildNode`s, and stages current-format TurboQuant
  pages through the existing page writer.

Validation:
- `cargo test concurrent_dsm_graph_single_participant_stages_current_format_pages -- --nocapture`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether exposing page staging from externally assembled graph nodes is the
  right boundary for the concurrent DSM graph path.
- Whether `insert_concurrent_dsm_graph_participant` should stay this small
  wrapper around the deterministic partition helper or carry more worker-state
  validation now.
- Whether the leader-only proof is sufficient before wiring real participant
  workers into the parallel build coordinator.
