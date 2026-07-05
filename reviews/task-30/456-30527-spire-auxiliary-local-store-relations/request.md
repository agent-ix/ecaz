# Review Request: SPIRE Auxiliary Local Store Relations

## Checkpoint

- Code commit: `4be77293`
  (`Create SPIRE auxiliary local store relations`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 partition-store relation layout and relation-backed multi-store build publication

## Summary

Phase 4 multi-store builds now create physical auxiliary local-store relations
instead of publishing every logical store against the root/control index
relation.

This checkpoint:

- creates the planned bounded `ec_spire_store_<index_oid>_<store_id>` store
  relations for multi-store populated builds;
- preserves the single-store compatibility path where store 0 remains the
  root/control index relation;
- creates auxiliary stores in the resolved per-store tablespace, preserving
  repeated tablespace OIDs for same-device baseline runs;
- initializes each auxiliary store relation with the SPIRE root/control
  metadata block so existing object tuple append/read helpers can use it;
- records internal catalog dependencies from each auxiliary store relation to
  the owning root/control index relation;
- publishes the created store relids in the active `SpireLocalStoreConfig`;
- updates the hash-routed build PG18 fixture to assert distinct physical
  `store_relid` values and the presence of created auxiliary heap relations.

The first implementation attempt used `RELKIND_INDEX` sidecar relations, but
PostgreSQL rejected that shape because there is no matching `pg_index` row. The
landed checkpoint uses internal heap relations with heap table AM storage,
while SPIRE still writes and reads raw object pages through the existing
buffer/page helpers rather than heap tuples.

## Files

- `src/am/ec_spire/storage/relation_plan.rs`
- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/build/tuples.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

Please review:

- whether the `heap_create_with_catalog` arguments are appropriate for
  internal SPIRE sidecar storage;
- whether the internal dependency direction is correct for root index drop
  lifecycle;
- whether initializing the store relation metadata page immediately after
  catalog creation is sufficient before object writes;
- whether the build-time lock and command-counter sequence is defensible;
- whether the single-store root/control compatibility path remains intact;
- whether any non-build paths still assume `store_relid == index_relid`.

## Validation

- `cargo test local_store_relation_plan --lib`
- `cargo pgrx test pg18 test_ec_spire_populated_build_hash_routes_logical_store_set`
- `cargo pgrx test pg18 test_ec_spire_relation_storage_snapshot_sql`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

No measurement claims are made in this packet.
