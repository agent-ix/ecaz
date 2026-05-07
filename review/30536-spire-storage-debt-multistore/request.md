# Review Request: SPIRE Multi-Store Storage Debt Diagnostics

- Code commit: `db515b69` (`Aggregate SPIRE storage debt across stores`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local placement
- Agent: coder1

## Summary

This checkpoint fixes the relation storage-debt diagnostic for Phase 4
multi-store indexes.

Before this change, `ec_spire_index_relation_storage_snapshot` scanned only the
root/control index relation. In a multi-store index, the active placement
directory can point at auxiliary local store relations, so the diagnostic could
miss object tuples, block counts, and cleanup-candidate debt outside the root
relation.

The diagnostic now:

- keys active tuple references by `(store_relid, ItemPointer)` rather than TID
  alone, so equal block/offset pairs in different store relations do not
  collide;
- includes root/control manifest and placement-directory tuples from the root
  relation;
- discovers all store relation OIDs from the active placement directory;
- opens auxiliary store relations under `AccessShareLock`, scans their object
  tuples, and closes them deterministically;
- sums relation block count, object tuple count/bytes, active referenced
  tuple count/bytes, and cleanup-candidate tuple count/bytes across the full
  active local store set.

The existing two-store PG18 fixture now asserts that storage diagnostics see
the auxiliary store relations, report no cleanup candidates immediately after
build, and report cleanup candidates after a post-build insert publishes a
new epoch.

## Review Focus

1. Confirm `(store_relid, ItemPointer)` is sufficient identity for active
   tuple tracking across root and auxiliary store relations.
2. Check that opening auxiliary store relations with `AccessShareLock` matches
   the diagnostic/read-only contract.
3. Verify that relation close happens even when `scan_object_tuples` returns a
   recoverable `Result` error.
4. Confirm the SQL test assertions cover the multi-store regression without
   turning this into a physical-reclamation implementation.
5. Check tracker wording: this improves cleanup-debt visibility but does not
   claim old-epoch tuple reclamation is implemented.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test relation_object_prefetch_groups --lib`
- `cargo pgrx test pg18 test_ec_spire_populated_build_hash_routes_logical_store_set`

PG17 was not run; this slice is local-store relation diagnostic coverage on
the PG18 Phase 4 branch.

## Notes

This keeps physical cleanup explicitly deferred. The point of this checkpoint
is that operators and future maintenance code can now see storage debt across
all local store relations, rather than undercounting debt after Phase 4
multi-store placement.
