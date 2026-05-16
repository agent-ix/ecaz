---
topic: spire-placement-planner-gate-index
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
stage: phase-12.3
status: open
---

# Review Request: SPIRE Placement Planner Gate Index

## Scope

Phase 12.3 code checkpoint for commit `77f2e302`
(`Index SPIRE placement planner gate`).

This slice replaces the DML PK-select CustomScan planner gate's
`ec_spire_placement` heap scan with a bounded indexed lookup:

- adds `ec_spire_placement_by_index_oid` to fresh bootstrap SQL and upgrade SQL;
- adds the same index in the 0.1.1 -> 0.1.2 migration for existing upgraded
  deployments;
- changes `custom_scan_index_has_sql_placement(index_oid)` from
  `table_beginscan_catalog` over `ec_spire_placement` to an explicit btree
  `index_beginscan` over `ec_spire_placement_by_index_oid`;
- keeps the planner gate fail-closed if the placement table or lookup index is
  missing;
- adds PG coverage that inserts unrelated placement rows and asserts an
  `index_oid` lookup uses `ec_spire_placement_by_index_oid`;
- keeps the existing DML PK-select CustomScan test passing, proving the planner
  still replaces eligible PK SELECTs after the lookup change;
- marks the first two Phase 12.3 planner-hardening rows complete in the task
  tracker.

## Files

- `ecaz--0.1.0--0.1.1.sql`
- `ecaz--0.1.1--0.1.2.sql`
- `sql/bootstrap.sql`
- `src/am/ec_spire/custom_scan.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

Packet-local logs are under `artifacts/`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo test custom_scan --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_placement_index_oid_lookup_uses_index_sql`
- `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_pk_select_customscan_local_sql`

No PG17 validation was run; this is Phase 12 PG18-primary planner hardening.

## Review Focus

- Confirm the new lookup is actually bounded by the dedicated `index_oid` index
  and does not regress the DML PK-select planner gate.
- Confirm the SQL migration shape is correct for both fresh installs and
  0.1.1 -> 0.1.2 upgrades.
- Confirm the test coverage is sufficient for the Phase 12.3 P2 tracker rows.
