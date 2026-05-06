# Review Request: SPIRE Auxiliary Store Autovacuum Guard

- Code commit: `9833a889` (`Guard SPIRE auxiliary stores from autovacuum`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local placement feedback cleanup
- Agent: coder1

## Summary

This checkpoint addresses the blocker from packet `30527` and the cleanup
items from the Phase 4 roll-up packet `30509`.

Auxiliary local-store relations are still PostgreSQL heap relations for catalog
and tablespace purposes, but SPIRE writes opaque partition-object bytes into
their pages. To keep autovacuum from interpreting those bytes as heap tuples,
the auxiliary relation creation path now passes a `text[]` reloptions datum to
`heap_create_with_catalog` with:

- `autovacuum_enabled=false`

The existing two-store PG18 build fixture now asserts both created
`ec_spire_store_*` auxiliary relations have that reloption in `pg_class`.

This checkpoint also:

- routes auxiliary store block-zero initialization through
  `initialize_aux_store_metadata_page`, avoiding a misleading root/control
  helper name at the aux-store call site;
- deletes the dead singular `prefetch_store_object_read_group` production
  helper and updates the unit test to call the plural helper with a one-element
  slice;
- records that packet `30530` measured the Phase 1 gate on single-store SPIRE,
  while packet `30533` covers same-device and `/mnt/e` two-store recall parity;
- removes the empty stale `review/30529-spire-phase1-recall-latency-gate/`
  directory from disk;
- tracks multi-store REINDEX as an explicit Phase 4 follow-up because
  auxiliary relation rebuild/retirement needs dedicated lifecycle semantics
  beyond internal catalog dependencies.

## Review Focus

1. Confirm that constructing `autovacuum_enabled=false` as a PostgreSQL
   `text[]` reloptions datum is the right `heap_create_with_catalog` boundary.
2. Check that the reloption is applied only to auxiliary local-store relations;
   the single-store path still uses the root/control index relation directly.
3. Verify the PG18 assertion against `pg_class.reloptions` is enough to prove
   the blocker fix without adding an invasive manual `VACUUM <auxrel>` guard in
   this checkpoint.
4. Confirm the aux-store metadata wrapper makes the block-zero intent clearer
   without changing the root/control page codec.
5. Review the REINDEX tracker/design wording: it should mark the unsupported
   lifecycle honestly without claiming it is implemented.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test prefetch_store_object_read_groups_prefetches_leaf_and_delta_routes --lib`
- `cargo test prefetch_store_object_read_groups_prefetches_every_store_before_scoring --lib`
- `cargo pgrx test pg18 test_ec_spire_populated_build_hash_routes_logical_store_set`

PG17 was not run; this feedback slice is centered on the PG18 Phase 4
multi-store relation creation path.

## Notes

This does not add a manual `VACUUM <auxrel>` rejection path. The blocker was
autovacuum accidentally visiting auxiliary stores, and the catalog reloption now
prevents that automatic path. A manual guard can be added later if we find a
clean AM-owned boundary for direct auxiliary-relation maintenance commands.
