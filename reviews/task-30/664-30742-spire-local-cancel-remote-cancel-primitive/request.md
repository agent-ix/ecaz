# Review Request: SPIRE Local-Cancel Remote-Cancel Primitive

## Summary

Code checkpoint: `be5be41b2768ff560652dc1b647d5cbdf45d6f80`

This slice starts Phase 11 C2 local-cancellation propagation without claiming
the production PostgreSQL interrupt bridge is complete.

- Added a `tokio-postgres` cancel-token wrapper around production transport
  probe and compact-candidate receive query futures.
- Added test-only local-cancel triggers that request remote query cancellation
  deterministically for PG18 loopback fixtures.
- Updated production executor state so a `local_query_cancelled` result from
  transport or candidate receive transitions all dispatches to
  `remote_executor_cancelled` instead of recording an ordinary per-node
  transport/receive failure.
- Updated the Phase 11 task/design docs to record this as the first C2
  primitive while keeping the real PostgreSQL interrupt bridge open.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
  - `run_query_with_optional_local_cancel`
  - local-cancel handling in `apply_transport_probe_rows` and
    `apply_candidate_receive_results`
  - executor state tests for global cancellation from transport and receive
- `src/lib.rs`
  - PG18 loopback tests for transport and compact-candidate receive cancel-token
    paths
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test --no-default-features --features pg18 production_executor_ --lib`
- `cargo pgrx test pg18 local_cancel`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`

## Review Questions

- Is the adapter-side cancel-token primitive appropriately scoped for the first
  C2 slice, with the real PostgreSQL interrupt bridge still marked open?
- Should `local_query_cancelled` continue to cancel all dispatches globally
  from both transport and compact-candidate receive outcomes?
- For the current per-query connection lifecycle, is it acceptable to request
  remote cancel and then close/abort the connection task rather than draining a
  cancelled query response?

