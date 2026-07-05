# 30727 — SPIRE Production Candidate Receive Adapter

## Summary

This checkpoint adds the first async compact-candidate receive adapter for the
production executor path. It remains test-facing and does not yet wire AM scan
production execution.

Code commit: `25d8f0e59eeeeae2a56c2c0483a180ba4901c5cc`

Changes:

- Adds `SpireRemoteProductionCandidateReceiveRequest` and
  `SpireRemoteProductionCandidateReceiveResult`.
- Extends the narrow `tokio-postgres` adapter with
  `run_candidate_receive_requests(...)`.
- Executes the existing `ec_spire_remote_search(...)` SQL template over the
  async adapter and decodes rows with the existing candidate decoder.
- Reuses `validate_remote_search_candidate_batch(...)` before returning a
  candidate batch.
- Returns per-node failure categories for invalid parameters, conninfo parse,
  connect, statement-timeout setup, remote query, decode, and validation
  failures.
- Adds PG18 loopback coverage proving the async receive adapter can return a
  validated compact candidate batch from a real `rabitq` remote index.
- Updates the Phase 11 task file while keeping the AM scan integration checkbox
  open.

## Validation

Packet-local logs are under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_production_candidate_receive_loopback`
- `git diff 8a9f8781e5f14379511d7d803e8c6d7ece406deb 25d8f0e59eeeeae2a56c2c0483a180ba4901c5cc --check`

## Review Questions

1. Is this the right adapter boundary for compact candidate receive before AM
   scan wiring?
2. Should candidate decode and batch validation failures remain separate
   failure categories for C4 strict/degraded mapping?
3. Is the PG18 loopback `rabitq` fixture enough for this test-facing receive
   slice, with multi-node AM scan merge left to the next packet?
