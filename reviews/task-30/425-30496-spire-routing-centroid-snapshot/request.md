# Review Request: SPIRE Routing Centroid Snapshot

Head SHA: `84dd1bd2`

## Summary

SPIRE now exposes relation-backed routing centroids through SQL with
`ec_spire_index_routing_centroid_snapshot(index_oid)`.

The snapshot walks active root and internal routing objects, emitting one row
per parent-to-child centroid edge. Each row includes parent PID/kind/level,
centroid ordinal and dimensions, child PID/kind/level, child placement state,
and the persisted centroid vector as `real[]`.

The new PG18 smoke verifies an empty index returns no rows, then builds a
`recursive_fanout = 2` hierarchy and confirms:

- six centroid rows are visible: two root-to-internal and four
  internal-to-leaf;
- root/internal parent row counts match the hierarchy shape;
- internal/leaf child row counts match the hierarchy shape;
- centroid array lengths match `centroid_dimensions`;
- child parent links match the emitted parent PID.

## Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test routing_centroid_snapshot_sql -- --nocapture`
  - 1 passed, including PG18 pg-test
    `pg_test_ec_spire_routing_centroid_snapshot_sql`.
- `cargo fmt`
- `git diff --check`

## Review Focus

- Confirm reading centroid vectors back from active routing objects is the right
  durable source for the Phase 3 centroid diagnostic.
- Confirm the SQL columns expose enough parent/child context for recursive
  hierarchy inspection.
- Confirm the test coverage is narrow but sufficient for this diagnostic
  surface.
