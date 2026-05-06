# Review Request: SPIRE Mutation Local Store Routing

- Code commit: `70a94d6c` (`Route SPIRE mutations through local stores`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local placement
- Agent: coder1

## Summary

This checkpoint routes relation-backed SPIRE mutation paths through the active
local store set instead of the root index relation store:

- live insert loads the active placement snapshot, routes the inserted vector to
  its base leaf, and writes the insert delta into that base leaf's local store;
- vacuum bulkdelete and live-assignment counting read through store sets opened
  from the active placement directory;
- delete-delta publication writes each delete delta into the matching base
  leaf's local store;
- delta compaction opens the writable store set from the active local-store
  config before reading and rewriting leaf objects.

`SpireRelationObjectStoreSet` now has a placement-addressed delta write helper
so mutation code can preserve the current design rule: deltas are colocated with
the parent/base leaf placement.

## Review Focus

1. Confirm that insert/delete/compaction mutation paths no longer fall back to
   `SpireRelationObjectStore::for_index_relation` for multi-store indexes.
2. Check the new placement-addressed writable lookup in
   `SpireRelationObjectStoreSet`, especially config validation and error
   reporting when a store is missing.
3. Verify that colocating deltas with the base leaf placement is still the right
   Phase 4 invariant for scan grouping and future cleanup.
4. Check whether the real-relation PG test extension is enough coverage for
   this slice before broader vacuum/multi-store coverage lands.

## Validation

- `cargo pgrx test pg18 test_ec_spire_populated_build_hash_routes_logical_store_set`
- `cargo test local_store_relation_plan --lib`
- `cargo fmt --check`
- `git diff --check`

PG17 was not run; this checkpoint is PG18-primary under the task policy and the
changed behavior is not PG17-specific.

## Notes

The PG18 test now inserts into a populated two-store `ec_spire` index and then
asserts:

- exactly one delta object is published after the insert;
- placements still reference exactly the original two store relids, proving the
  insert path did not create or write a root-relation fallback placement;
- an ordered scan after the insert returns all five rows.

This packet intentionally does not address the separate scan hot-path note from
`30519` about duplicate `require_lookup` calls; that is better bundled with the
parallel local fetch slice.
