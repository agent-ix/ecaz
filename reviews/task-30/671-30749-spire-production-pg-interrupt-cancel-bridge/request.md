# Review Request: SPIRE Production PG Interrupt Cancel Bridge

## Summary

Code checkpoint: `7af6fb0d32aca62d4f5062b806d87896920f02b1`

This slice moves the remote transport local-cancel path from a test-only timer
toward the production PostgreSQL backend signal path:

- Added a `SpireRemoteLocalCancelSource` abstraction so production transport and
  compact candidate receive use PostgreSQL interrupt polling by default, while
  deterministic timer cancellation remains test-only.
- Added dynamic backend symbol lookup for `InterruptPending` and
  `QueryCancelPending` so the adapter can poll real PostgreSQL backend cancel
  flags without adding link-time references that break the pgrx test binary.
- When the local backend has query-cancel flags pending, the adapter now sends
  the remote `tokio-postgres` cancel token and reports
  `local_query_cancelled`.
- Added a focused PG18 test that sets/restores those backend flags inside a
  `pg_test` backend and proves the default production probe path maps them to
  `remote_transport_failed` / `local_query_cancelled`.
- Updated the Phase 11 task and coordinator/executor design docs to record this
  as the first production interrupt bridge, while keeping C2 open for distinct
  local statement-timeout provenance.

This does not claim full C2 completion. The remaining production blocker is to
distinguish local query cancellation from local statement timeout before the AM
bridge can expose final strict/degraded fault-matrix behavior.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 prod_transport_pg_interrupt_bridge_cancel`
- `cargo pgrx test pg18 local_cancel_remote_cancel`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`

## Review Questions

- Is dynamic `dlsym` lookup of PostgreSQL backend signal flags the right boundary
  for this pgrx extension, given direct static references to those symbols fail
  in the pgrx test binary?
- Is the `SpireRemoteLocalCancelSource` split narrow enough to keep timer-based
  cancellation test-only while making production paths poll PostgreSQL state by
  default?
- Is it acceptable to land query-cancel provenance first and leave local
  statement-timeout classification as the explicit follow-up before closing C2?
