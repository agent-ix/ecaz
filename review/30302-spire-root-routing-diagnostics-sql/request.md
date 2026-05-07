# SPIRE Root Routing Diagnostics SQL

## Checkpoint

- Code commit: `c8edc1ea` (`Expose SPIRE root routing diagnostics`)
- Feedback response commit: `6ff7fd33`
  (`Address SPIRE root routing review nits`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: active root routing diagnostics for relation-backed `ec_spire`
  snapshots

## Summary

This checkpoint adds a SQL diagnostic surface for the active root routing
object:

- Added `ec_spire_index_root_routing_snapshot(index_oid)` as a stable, strict
  SQL table function for `ec_spire` indexes.
- The function validates the supplied OID as an `ec_spire` index, reads root
  control, returns no rows for empty indexes with no active epoch, and resolves
  the active relation-backed epoch manifests.
- The AM helper finds the active root partition object, reads its routing
  children, and returns one row per root centroid-to-child PID edge.
- Each row reports active epoch, root PID/object version/level/child count,
  centroid dimensions and ordinal, child PID/object version/kind/level/parent
  PID, assignment count, placement state, object bytes, and local store
  identity.
- The Task 30 plan now records that active root routing rows are visible for
  the local single-store path.

This does not implement recursive multi-level SPIRE routing, local multi-store
placement, remote placement maps, replica reads, physical old-epoch cleanup,
real SQL `VACUUM` end-to-end validation, recall/latency summary evidence, or
PQ-FastScan scorer binding.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_root_routing_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1088 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `208 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a measurement or recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- The root routing rows are active snapshot diagnostics, not persistent query
  telemetry and not PostgreSQL declarative table partition metadata.

## Feedback Response

Reviewer feedback in `feedback.md` requested a checked root child-count
conversion, a short note explaining why the root search walks the whole
manifest, and a trivial assertion for `child_store_relid`. Response commit
`6ff7fd33`:

- Replaced the `usize` to `u64` cast for `root_child_count` with
  `u64::try_from(...)` and an explicit diagnostic error.
- Added a short comment that the manifest walk intentionally continues so
  malformed multiple-root epochs are reported.
- Extended the SQL test to assert the local single-store fixture reports one
  distinct `child_store_relid`.

Additional validation for `6ff7fd33`:

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_root_routing_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1094 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `214 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before response commit
