# Review Request: SPIRE Placement Write Contention

Code checkpoint: `ff28a1fd8cf7ab6dce05ec070900de1d3ec59102` (`Add SPIRE placement write contention fixture`)

## Scope

- Adds `test_pg18_ec_spire_placement_write_contention_distinct_pk_dml`.
- The fixture creates a SPIRE-indexed table, seeds `ec_spire_placement`, then
  releases 8 psql workers behind an advisory barrier.
- Each worker runs a distinct-PK INSERT plus a distinct-PK DELETE in one
  transaction while also inserting/deleting the corresponding placement rows.
- The fixture asserts worker success, inserted-row visibility through PK point
  lookups, final placement row count, no ungranted `ec_spire_placement` locks,
  unchanged `pg_stat_database.deadlocks`, and p99 completion below the
  predeclared 20s threshold.
- The Phase 12.4 tracker now records the fixture and the current partitioning
  decision: keep the shared `ec_spire_placement` table unless future
  packet-local evidence crosses the threshold or shows placement-page lock
  waits/deadlocks.
- Also records reviewer feedback from packet 30967 that the final readiness
  bundle should include endpoint tuple-transport readiness in the header.

## Validation

- `git diff --check ff28a1fd^ ff28a1fd`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_pg18_ec_spire_placement_write_contention_distinct_pk_dml`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and result lines.

## Review Focus

- Confirm the 8-writer / 20s threshold is a reasonable local contention gate
  for Phase 12.4.
- Confirm the fixture shape is sufficient for the H12 placement-table
  contention row, including both app-table distinct-PK DML and direct
  placement-table write pressure in the same transactions.
- Confirm the tracker decision to defer `index_oid` partitioning is supported
  by this local evidence.
