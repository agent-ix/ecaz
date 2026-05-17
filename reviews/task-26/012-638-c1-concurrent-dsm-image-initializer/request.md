# Review Request: Concurrent DSM Image Initializer

Current head: `1b70413`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- ADR-048 now targets concurrent graph insertion into a DSM-resident graph.
- Packets 634 through 637 added the graph layout, node slot plan, compact code
  corpus, and preassembly plan.
- The next unsafe boundary is turning that pure plan into an initialized DSM
  memory image.

Change:
- Added `EcHnswConcurrentDsmGraphParts`, a pointer view over the initialized
  DSM sections:
  - header
  - node array
  - flat neighbor-slot array
  - compact code bytes
- Added `concurrent_dsm_graph_parts` to derive those section pointers from the
  layout offsets.
- Added `initialize_concurrent_dsm_graph_image` to write:
  - graph header, using `u32::MAX` as the invalid entry/node sentinel
  - per-node metadata from the node layout plan
  - per-node uninserted state
  - all neighbor slots initialized to the invalid-node sentinel
  - packed code bytes copied from the preassembly code corpus
- Kept LWLock initialization as an injected callback. This proves the memory
  image without hard-coding the final production tranche registration strategy.

Validation:
- `cargo test concurrent_dsm_graph_image`
- `cargo test concurrent_dsm`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether the pointer-view and initializer are the right boundary before DSM
  allocation is wired into `ParallelContext`.
- Whether the invalid sentinel and uninserted state representation are suitable
  for the upcoming insertion protocol.
- Whether keeping LWLock initialization callback-injected is the right way to
  avoid prematurely choosing a tranche strategy in this checkpoint.
