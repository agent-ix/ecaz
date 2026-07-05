# Review Request: Concurrent DSM Code Corpus

Current head: `9e441ec`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- ADR-048 requires worker processes to score HNSW candidates without reading
  the leader-local Rust `BuildState`.
- Packet 634 sized a compact code-corpus region in the concurrent DSM graph
  layout.
- Packet 635 added the node slot plan that future DSM initialization will use.

Change:
- Added `EcHnswConcurrentDsmCodeCorpus`, a flat fixed-width encoded-code corpus
  derived from `BuildTuple` rows.
- The corpus records:
  - node count
  - fixed code length
  - flat code bytes in node-index order
- Added `code_for_node` for future worker scoring and test-time validation.
- Added tests for:
  - fixed-width packing and per-node lookup
  - empty input
  - variable-width code rejection

Validation:
- `cargo test concurrent_dsm_code_corpus`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether this flat corpus is the right pre-DSM representation for copying
  compact codes into shared memory.
- Whether the corpus should carry more scoring metadata now (`dimensions`,
  `bits`, `seed`) or leave those in the future graph insertion context.
- Whether source-scored build handling needs an explicit guard at this layer,
  or only at the `ConcurrentDsm` plan-selection/wiring layer.
