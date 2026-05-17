# 30772 - SPIRE Stage E Lifecycle Matrix

## Summary

This packet reviews commit `b589f63f9b0646ae6a5af1ef8a81d7d8a9ad064c`
(`Define SPIRE Stage E lifecycle matrix`).

The slice defines online remote-index lifecycle behavior before implementing
the local multi-instance DDL/fault fixture.

Changes:

- Adds `ec_spire_remote_search_stage_e_lifecycle_matrix()`, a SQL-visible
  contract matrix for Stage E lifecycle cases.
- Covers `DROP INDEX` before fanout and after fanout/before receive.
- Covers `REINDEX INDEX CONCURRENTLY` before fanout and in flight, requiring
  remote index identity mismatch detection.
- Covers `CREATE INDEX CONCURRENTLY` as a new descriptor that must be deferred
  for existing fanout, plus the missing-descriptor case before registration.
- States strict/degraded actions, statuses, required detection, next executor
  step, and required fixture evidence for each lifecycle case.
- Registers the lifecycle matrix in the operator entrypoint contract.
- Updates Phase 11 to mark the lifecycle behavior definition done while
  keeping packet-local DDL fixture evidence open.

## Key Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `cargo pgrx test pg18 test_ec_spire_stage_e_lifecycle_matrix_contract`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`

All commands passed.

## Review Focus

- Are the DROP, REINDEX CONCURRENTLY, and CREATE INDEX CONCURRENTLY cases split
  at the right timing boundaries for the later fixture?
- Are strict/degraded actions and statuses correct, especially deferring new
  CREATE INDEX CONCURRENTLY descriptors for already-planned fanout?
- Are `remote_index_unavailable`, `endpoint_identity_mismatch`, and
  `requires_remote_node_descriptor` the right required detections?
- Is the Phase 11 task update scoped honestly, with DDL fixture logs still
  open?
