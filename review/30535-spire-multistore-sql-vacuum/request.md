# Review Request: SPIRE Multi-Store SQL VACUUM Coverage

- Code commit: `656f1de0` (`Cover SPIRE multistore SQL vacuum`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local placement
- Agent: coder1

## Summary

This checkpoint adds focused PG18 coverage for the relation-backed multi-store
mutation and cleanup path under PostgreSQL's real SQL `VACUUM` callback flow.

The new test `test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores`:

- builds a two-store `ec_spire` index with repeated `pg_default` tablespaces,
  preserving the same-device baseline configuration Phase 4 now supports;
- inserts a row after build, deletes an existing row, and runs `VACUUM` through
  an external `psql` connection so PostgreSQL drives the callbacks normally;
- asserts active placement diagnostics still span both local store ids and
  both store relation OIDs after cleanup;
- asserts delta objects/assignments are compacted away;
- disables seqscan and verifies ordered scan hides the deleted row while the
  inserted row remains fetchable.

The tracker now moves T30 from 86% to 90% and records the remaining Task 30
gates as PQ-FastScan scorer binding plus physical object reclamation/old-epoch
cleanup.

## Review Focus

1. Confirm the same-device two-store setup is the right fixture for callback
   coverage without depending on `/mnt/e` or cloud hardware.
2. Check that using external `psql` for INSERT/DELETE/VACUUM is consistent
   with the existing SQL VACUUM tests and genuinely exercises PostgreSQL's
   callback path.
3. Verify the post-vacuum assertions cover the important Phase 4 invariants:
   both store relations remain referenced, deltas are compacted, the deleted
   row is not returned, and the inserted row is still retrievable.
4. Confirm the tracker wording does not overclaim performance or production
   multi-NVMe behavior.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo pgrx test pg18 test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores`

PG17 was not run; this is a PG18 SQL `VACUUM` callback coverage slice and the
test is guarded by `#[cfg(feature = "pg18")]`.

## Notes

This packet is intentionally a coverage checkpoint rather than another storage
rewrite. It closes the broader multi-store write+fetch/vacuum gate left after
`30534` without creating another benchmark or production hardware claim.
