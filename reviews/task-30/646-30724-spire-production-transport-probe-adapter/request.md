# 30724 — SPIRE Production Transport Probe Adapter

## Summary

This checkpoint lands the first C1 transport-adapter slice for Phase 11 Stage C.

Code commit: `33796ac1beae8350c82740f47d07ea4e1d3217ce`

Changes:

- Adds direct `tokio`, `tokio-postgres`, and `futures-util` dependencies already
  present in the lockfile transitively.
- Adds a narrow `SpireRemoteProductionTransportAdapter` boundary for production
  fanout probes, separate from the existing blocking diagnostic
  `postgres::Client` receive path.
- Adds `SpireRemoteProductionTransportProbeRequest` and
  `SpireRemoteProductionTransportProbeRow` test-facing structs.
- Adds a PG18 loopback fixture proving a slow remote probe does not serialize a
  fast remote probe behind it.
- Processes the 30722 P2 review item by pinning the C1 connection lifetime:
  per-query connect / per-dispatch close. Pooling remains deferred until after
  cancellation and strict/degraded semantics are stable.
- Adds the 30722 P3 runbook surface cross-reference while updating the Phase 11
  task file.

This does not yet wire the adapter into compact candidate receive or the AM scan
production path. The parent Stage C transport task remains open until that
follow-up lands.

## Validation

Packet-local logs are under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_production_transport_probe_overlaps_ready_remotes`
- `git diff 96c4694f36e73a722e7db6bd4618894a1da1f1a5 33796ac1beae8350c82740f47d07ea4e1d3217ce --check`

## Review Questions

1. Is the `tokio-postgres` adapter boundary acceptable as the C1 overlapped
   transport proof while direct libpq async/pipeline FFI remains unavailable
   through `pgrx::pg_sys`?
2. Does the per-query connect / per-dispatch close contract close the 30722 P2
   concern cleanly enough for the C2 cancellation work to target?
3. Is the PG18 slow/fast loopback fixture strong enough to lock the
   non-serialized progress guarantee for this adapter slice, given that
   candidate merge and AM scan wiring are deliberately deferred?
