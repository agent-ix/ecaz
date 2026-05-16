# Review Request: SPIRE AM Materialized Remote Row

## Summary

This packet lands the first end-to-end Stage D SQL proof requested by reviewer
direction packet `30800`.

The new PG18 fixture creates a loopback remote-serving SPIRE index and a
coordinator SPIRE index whose selected leaves are remote-owned, registers a
remote node descriptor, resolves a remote heap candidate, registers a
coordinator materialization mapping through
`ec_spire_register_remote_row_materialization(...)`, and then issues a real
PostgreSQL executor query:

`SELECT id FROM ... ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1`

The query returns the materialized coordinator row through
`amrescan`/`amgettuple`.

The fixture exposed and this change fixes a production AM blocker before the
scan cursor: PostgreSQL planning calls SPIRE cost/diagnostic paths before
`amrescan`, and those paths still assumed remote-owned placements were
unreadable. Planner/diagnostic metadata now loads the coordinator fanout
manifest and reads the coordinator's published metadata copy while preserving
remote placement ownership for execution.

## Files

- `src/lib.rs`
- `src/am/ec_spire/diagnostics.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are under `artifacts/` and described in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/diagnostics.rs src/am/ec_spire/root/hierarchy_snapshots.rs src/am/ec_spire/root/snapshots.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo pgrx test pg18 test_ec_spire_prod_scan_am_delivers_materialized_remote_row`
- `cargo test snapshot_diagnostics --no-default-features --features pg18`

## Reviewer Focus

- Confirm the metadata-read adjustment is scoped correctly: planning and
  diagnostics may read the coordinator metadata copy for remote-owned
  placements, but execution ownership remains remote.
- Confirm the PG18 fixture is a sufficient explicit-register SQL proof for the
  first `30800` action item.
- Confirm this packet does not claim Stage D complete; the operator-owned
  mirror sync mechanism and catalog lifecycle coverage remain open.
