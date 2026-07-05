# Review Request: Concurrent DSM Node Slot Plan

Current head: `4cc3df3`

Scope:
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- ADR-048 targets concurrent graph insertion into a DSM-resident graph.
- Packet 633 pre-computed native build levels.
- Packet 634 added the C-compatible graph/header/node DSM layout.
- This packet keeps the next step pure and testable before the later unsafe DSM
  initializer writes node headers, LWLocks, atomics, and slot arrays.

Change:
- Added `EcHnswConcurrentDsmNodeLayout`, one row per graph node with:
  - node level
  - flat neighbor-slot offset
  - flat neighbor-slot count
- Added `EcHnswConcurrentDsmNodeLayoutPlan` to derive all node slices from the
  pre-computed `NativeBuildLevels` and `m`.
- Updated `EcHnswConcurrentDsmGraphLayout` sizing to consume the same node-plan
  total slot count, so future DSM initialization and DSM sizing share one slot
  accounting path.
- Added tests for:
  - level plan `[0, 2]`, `m = 2` -> node slot slices `[0..4]` and `[4..12]`
  - empty level plans

Validation:
- `cargo test concurrent_dsm_node_layout_plan`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether this node-plan type is the right shared source of truth for both DSM
  sizing and future DSM pointer initialization.
- Whether slot counts should remain expressed as flat `u32` counts here, or
  whether the plan should already expose per-layer ranges for worker insertion.
- Whether any additional invariants should be recorded before adding the unsafe
  DSM initializer.
