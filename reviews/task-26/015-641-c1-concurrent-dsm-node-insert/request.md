# Review Request: Concurrent DSM Node Insert

## Branch / Commit

- Branch: `task19-parallel-index-build`
- Code commit: `b949614` (`Insert nodes into concurrent DSM graph`)

## Context

- ADR-048 now targets concurrent graph insertion into a DSM-resident HNSW graph.
- Packets 638-640 established the DSM image initializer, static insertion
  ranges, and completed-graph readback into the existing page staging type.
- The next blocker is the worker-callable insertion core: search ready DSM
  nodes, write forward slots, and apply backlinks under node locks.

## Changes

- Added `EcHnswConcurrentDsmInsertConfig`, per-participant insertion scratch,
  and lock-operation hooks.
- Added `insert_concurrent_dsm_graph_node(...)`:
  - skips the preinitialized fixed entry node
  - transitions nodes through `UNINSERTED -> INSERTING -> READY`
  - searches only `READY` DSM nodes from the fixed entry point
  - writes selected forward slots under the inserted node lock
  - applies backlink mutations to selected neighbors under target node locks
- Reused the existing layer slot bounds and backlink candidate pruning helpers
  so DSM insertion preserves the native HNSW slot layout.
- Added tests proving:
  - a DSM node insert writes forward slots and backlinks
  - the fixed entry node is not reinserted
  - existing DSM layout/readback tests still pass with the new state.

## Validation

- `cargo test concurrent_dsm -- --nocapture`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

## Review Focus

- Whether the lock boundary is correct: node state and forward slots are updated
  under the node's exclusive LWLock; successor reads copy neighbor indexes under
  shared lock before scoring.
- Whether publishing the node as `READY` before backlink writes is acceptable
  for HNSW build semantics, matching the same eventual-backlink model used by
  live insert.
- Whether this insertion core is the right boundary before wiring the worker
  callback and leader participant loop.
