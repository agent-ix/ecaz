# Review Request: Concurrent DSM Preassembly Plan

Current head: `92aa0fc`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- Packet 633 added native build-level precomputation.
- Packet 634 added DSM graph layout sizing.
- Packet 635 added flat node neighbor-slot planning.
- Packet 636 added compact encoded-code corpus packing.

Change:
- Added `EcHnswConcurrentDsmPreassemblyPlan`, composing:
  - `NativeBuildLevels`
  - `EcHnswConcurrentDsmNodeLayoutPlan`
  - `EcHnswConcurrentDsmCodeCorpus`
  - `EcHnswConcurrentDsmGraphLayout`
- `for_state` now validates that:
  - source-scored builds are rejected for the future concurrent DSM path
  - graph layout node count matches the code corpus node count
  - graph layout neighbor-slot count matches the node layout plan
- Empty build state is handled explicitly without asking level precompute for a
  seed.
- Added tests for populated, empty, and source-scored guard cases.

Validation:
- `cargo test concurrent_dsm_preassembly_plan`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether this preassembly object is the right boundary before unsafe DSM
  allocation/initialization.
- Whether source-scored build rejection belongs here, or should only live at
  plan-selection time.
- Whether additional scoring metadata should be included before wiring the
  worker insertion context.
