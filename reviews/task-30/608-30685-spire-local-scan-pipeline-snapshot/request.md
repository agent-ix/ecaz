# Review Request: SPIRE Local Scan Pipeline Snapshot

Code checkpoint: `55326c65` (`Add SPIRE local scan pipeline snapshot`)

## Scope

- Advances Phase 10.7 by adding
  `ec_spire_index_scan_pipeline_snapshot(index_oid, query)`.
- Orders local scan diagnostics into routing, placement, prefetch, candidates,
  heap rerank, and remote-fanout rows, mirroring the operator shape of
  `ec_spire_remote_pipeline_steps`.
- Reuses existing routing and placement diagnostic collectors; no timing or
  recall claim is made.
- Exposes planned heap-rerank row count from candidate winners plus effective
  rerank width, and keeps local remote-fanout count explicitly zero.
- Marks the Phase 10.7 local scan pipeline snapshot checklist item complete.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_scan_pipeline_snapshot_sql --lib`

## Review Focus

- Confirm the step ordering and counters are useful for packet-local Phase 10
  diagnostics.
- Confirm deriving heap-rerank row count from winners and effective rerank width
  is clear enough as a planned I/O count, not a latency measurement.
- Confirm keeping `remote_fanout` present with zero count is the right local
  mirror of the remote pipeline shape.
