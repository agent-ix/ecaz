# 30725 — SPIRE Production Transport Failure Isolation

## Summary

This checkpoint hardens the C1 transport probe adapter so a single failed remote
does not abort the whole fanout batch.

Code commit: `5ee88b3bc170b700ce051610e21a631efc3b0dc6`

Changes:

- Converts per-node conninfo parse, connect, statement-timeout setup, and remote
  query failures into `SpireRemoteProductionTransportProbeRow` values.
- Keeps runtime creation as the only batch-level adapter error.
- Adds stable failure categories:
  `conninfo_parse_failed`, `connect_failed`,
  `statement_timeout_setup_failed`, and `remote_query_failed`.
- Keeps the successful remote path returning `status = ready` and
  `failure_category = none`.
- Adds PG18 coverage where one remote uses a missing local socket and another
  uses the loopback pg_test connection; the ready remote must still complete.
- Updates the Phase 11 task file to record per-node transport failure rows as a
  completed C1 hardening item.

This still does not wire compact candidate receive or AM scan production state.
The next implementation slice remains the production state/receive integration.

## Validation

Packet-local logs are under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 production_transport_probe`
- `git diff a62c82f383657fe0f1760dea8e1731ab51687cd7 5ee88b3bc170b700ce051610e21a631efc3b0dc6 --check`

## Review Questions

1. Is keeping runtime creation as the only batch-level failure the right
   adapter contract, with all per-node failures returned as rows?
2. Are the initial failure categories specific enough for the upcoming
   strict/degraded state machine?
3. Is the missing-socket plus loopback PG18 fixture adequate coverage that one
   failed remote cannot suppress a ready remote in the transport adapter?
