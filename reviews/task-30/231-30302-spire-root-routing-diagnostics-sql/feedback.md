# 30302 SPIRE Root Routing Diagnostics SQL — review

Code commit `c8edc1ea`. Read `src/am/ec_spire/mod.rs:505-577`,
`src/lib.rs:1237-1294,3267-3338`, and the task plan diff. Cross-referenced
`SpireRoutingPartitionObject::children()` in `storage.rs:1361-1375` and
`SpireValidatedEpochSnapshot::new` in `meta.rs:977-988`.

## What landed

- `SpireIndexRootRoutingSnapshotRow` shape (`mod.rs:151-170`): per-centroid
  row carrying root identity (`root_pid`, `root_object_version`,
  `root_level`, `root_child_count`, `centroid_dimensions`), the per-edge
  `centroid_index` ordinal, and child identity + placement metadata
  (`child_kind`, `child_object_version`, `child_level`,
  `child_parent_pid`, `child_assignment_count`, `child_node_id`,
  `child_local_store_id`, `child_store_relid`, `child_placement_state`,
  `child_object_bytes`). 18 columns total at the SQL surface.
- Two new shared name helpers — `placement_state_name` (4 states) and
  `partition_object_kind_name` (4 kinds) — both `&'static str`-returning,
  so per-row enum→label conversion stays branch-free.
- `index_root_routing_snapshot` in `mod.rs:507-577`:
  - `active_epoch == 0` short-circuits to empty rows before opening the
    object store.
  - Constructs a `SpireValidatedEpochSnapshot::new(...)` directly from
    raw manifests — different style than the placement / scan diagnostic
    surfaces, which use `SpirePublishedEpochSnapshot::new` then
    `from_snapshot`. The new path internally calls the published
    constructor + `from_snapshot`, so behavior is equivalent.
  - Walks all manifest entries, reads each header, and identifies the
    Root kind. Errors with
    `"found multiple root objects"` if a second Root is encountered;
    errors with `"found no active root object"` if `active_epoch > 0`
    but the manifest has no Root.
  - Iterates `root_object.children()` (centroid_index, child_pid,
    centroid slice triple) and for each child does a
    `snapshot.require_lookup(child_pid, "root routing child")` plus a
    header read against the relation object store, building one row per
    edge. If any child lookup or header read fails, the whole call
    fails — partial rows are dropped.
- pgrx wrapper `ec_spire_index_root_routing_snapshot` in
  `lib.rs:1237-1294` follows the same `AccessShareLock` →
  `index_open` → row-collect → `index_close` shape as the other
  ec_spire snapshot helpers. `i64::try_from(...).expect(...)` per
  64-bit column; `i32::from(...)` for `root_level` /
  `centroid_dimensions` (both `u16`); `i64::from(...)` for `u32`
  fields.
- Plan updates record this surface as PG18-covered for empty and
  populated local single-store indexes.

## Correctness

- `SpireValidatedEpochSnapshot::new` does the validate-then-build-pid-index
  work upfront, so `require_lookup` calls inside the children loop hit
  a populated index and report a friendly error string with the caller
  label `"root routing child"` — matches the lookup conventions in
  scan.rs.
- Multiple-root and no-root errors are real defensive checks: the
  manifest *should* have exactly one Root object per active epoch, and
  these errors would surface a manifest-shape bug as an SQL error
  rather than silently returning the first root's edges.
- `child_object_bytes` is from the placement entry, not a fresh object
  read, so this column is consistent with the values reported by
  `ec_spire_index_placement_snapshot`'s `placement_object_bytes` for
  the same PID.
- The walk reads each child's header even though `child.centroid_index`
  / `child.child_pid` are already in the parent's routing object. The
  header read is what populates `child_kind`, `child_object_version`,
  `child_level`, `child_parent_pid`, `child_assignment_count` — so
  one extra page read per child is the cost of those columns. Fine for
  a diagnostic; would matter on very wide roots if this were a hot
  path.
- `child_parent_pid` lets the SQL test cross-check that all children
  point back at `root_pid`. The PG18 test exercises this with
  `bool_and(child_parent_pid = root_pid)`.
- `centroid_dimensions` is constant per query but reported per row.
  Acceptable for a diagnostic surface (and the test uses `max()` to
  dedupe).

## Test coverage

- PG18 `test_ec_spire_root_routing_snapshot_sql`:
  - Empty index (`ec_spire_route_empty_idx` after `CREATE INDEX` over an
    empty table): asserts 0 rows from the SQL function.
  - Populated 2-vector / `nlists=2` build: asserts `row_count = 2`,
    `root_child_count = 2`, `centroid_dimensions = 2`,
    `count(*) WHERE child_kind = 'leaf' = 2`,
    `sum(child_assignment_count) = 2`,
    `bool_and(child_parent_pid = root_pid)` true. The aggregate
    cross-checks (sum of assignment counts matches inserted row count;
    every child links back to the root) are particularly good — they
    verify the diagnostic is internally consistent rather than just
    "at least one row exists".
- Gaps:
  - The "multiple Root" and "no Root despite `active_epoch > 0`" error
    paths are unexercised. Both require a malformed manifest, which
    the public path can't construct, so this is realistically only
    addressable via a unit test that builds a synthetic snapshot.
    Worth adding a unit-level test before this surface gets relied on
    for incident triage, since these errors are precisely what would
    fire in a real corruption scenario.
  - `child_kind = "internal"` rows untested — single-level only, so
    same status as the placement diagnostic.
  - `child_placement_state` other than `"available"` is unexercised.
    Same gap as elsewhere; would need a Degraded snapshot fixture.
  - `child_store_relid` is reported but not asserted in the test. It's
    a `u32` from `placement.store_relid`. For local single-store this
    will be the same value across all rows, so testing it'd be trivial
    (e.g. `count(DISTINCT child_store_relid) = 1`).

## Style / minor

- `mod.rs:545`: `let root_child_count = root_object.child_count() as u64;`
  uses an `as` cast on `usize → u64` rather than `u64::try_from(...)?`.
  Other counts in this file go through `try_from` with `.expect(...)` or
  `.ok_or_else(...)`. On 64-bit Postgres targets this is identical, but
  it breaks the pattern. Trivial fix.
- The choice of `SpireValidatedEpochSnapshot::new` here vs
  `SpirePublishedEpochSnapshot::new` + downstream `from_snapshot` in
  30299/30300 is stylistically inconsistent across the diagnostic
  surfaces. `::new` is shorter and probably the better default; if
  that's the direction, a follow-up could converge the placement /
  scan paths onto the same constructor.
- The "find the root" loop walks all manifest entries even after
  `root.is_some()` so the multiple-root check fires correctly. That's
  the right tradeoff. A short comment explaining "we walk the full
  manifest to detect multiple roots" would document the intent.

## Status

Lands cleanly. The surface is well-shaped — per-edge rows with both
parent and child identity, plus enough placement metadata to use this
as a starting diagnostic for routing-graph integrity. The PG18 test's
aggregate cross-checks are a good model for the other diagnostic
surfaces. Main thing I'd want before this is treated as a corruption-
debugging tool: a synthetic-snapshot unit test that exercises the
multiple-Root / no-Root error paths so we know those messages actually
surface as expected when something goes wrong.
