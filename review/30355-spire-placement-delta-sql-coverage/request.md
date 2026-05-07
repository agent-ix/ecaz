# 30355 SPIRE Placement Delta SQL Coverage

## Request

Review the placement diagnostics SQL coverage added for post-build delta
objects.

## Scope

- Extended `test_ec_spire_placement_snapshot_sql` after the populated
  single-store assertions.
- Inserted one row after index build to publish a delta placement.
- Asserted placement snapshot delta object count, assignment count, and delta
  object byte accounting at the SQL surface.
- Updated Task 30 status.

## Behavior Covered

After a post-build insert, `ec_spire_index_placement_snapshot(index_oid)` now
has PG18 coverage proving:

- placement count grows from root + leaves to root + leaves + delta
- `delta_object_count = 1`
- `assignment_count` includes the delta assignment
- `delta_object_bytes` is positive

This closes the single-level delta-kind gap for the placement snapshot surface;
Internal-kind and multi-store grouping remain future multi-level/multi-store
work.

## Validation

- `cargo fmt`
- `cargo test --lib test_ec_spire_placement_snapshot_sql --no-default-features --features pg18 -- --nocapture`
- `git diff --check`
