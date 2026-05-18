# 30726 — SPIRE Production Transport State

## Summary

This checkpoint wires C1 transport result rows into the production executor
state machine, without yet wiring compact candidate receive or the AM scan path.

Code commit: `394582e18d194d9a757e7d8064c2acccf83d6a2a`

Changes:

- Extends production dispatch state beyond dry `Planned` /
  `BlockedBeforeDispatch` to include `TransportReady` and `TransportFailed`.
- Adds transport counters to
  `ec_spire_remote_search_production_executor_state_summary(...)`:
  pending, sent, ready, failed, row count, and first failure category.
- Maps ready transport outcomes to
  `next_executor_step = compact_candidate_receive` and
  `status = requires_compact_candidate_receive`.
- Maps failed transport outcomes back to
  `next_executor_step = production_transport_adapter`,
  `status = remote_transport_failed`, and preserves the first failure category.
- Rejects transport rows that do not match a planned production dispatch.
- Keeps the public production-state SQL summary dry: it still performs no
  conninfo secret lookup and opens no sockets, but now reports C1 transport
  counters as zero or pending for dry state.
- Processes 30724 P2 by documenting that `tokio-postgres` is the current C1
  implementation, not the C0-C6 transport contract.

The Stage C parent task remains open: compact candidate receive and AM scan
production wiring are still deferred.

## Validation

Packet-local logs are under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test --no-default-features --features pg18 production_executor_state_`
- `git diff ca0faede68423565cea7204d391f77a0f29599cc 394582e18d194d9a757e7d8064c2acccf83d6a2a --check`

## Review Questions

1. Are `TransportReady` and `TransportFailed` the right state granularity before
   candidate receive lands?
2. Should transport failure dominate the summary status when any dispatch fails,
   or should partial ready/pending states be represented differently before C4
   strict/degraded semantics?
3. Are the new SQL summary counters sufficient for operators to distinguish dry
   pending state, transport-ready state, and transport-failed state?
