# SPIRE Placement Entry Locators

## Checkpoint

- Code commit: `115ae32f` (`Persist SPIRE placement entry locators`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: live relation publish helper for placement write evidence

## Summary

This checkpoint resolves the live relation-backed `placement_tid` shape needed before populated build publication:

- `SpireManifestEntry.placement_tid` is treated as a locator for a durable placement-entry tuple.
- Object tuple locators remain in `SpirePlacementEntry.object_tid`.
- The placement directory remains the query-time `pid -> local_store_id -> object` map and is still persisted as a manifest bundle component.
- A new `write_placement_entries_to_relation` helper writes each encoded placement entry to the index relation and returns `(pid, placement_tid)` evidence.
- A new `object_manifest_from_placement_writes` helper builds the object manifest from placement-directory entries plus those durable placement-entry locators.

This avoids overloading object tuple TIDs as placement row evidence and gives the publish coordinator concrete placement writes to validate before manifest encoding.

## Changed Files

- `src/am/ec_spire/build.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable `imports_granularity` / `group_imports`.
- `cargo test --lib object_manifest_from_placement_writes --no-default-features --features pg18`
  - `2 passed; 0 failed; 0 ignored; 0 measured; 1073 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `194 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This does not yet wire populated `ambuild` to relation-backed object/manifest publication.
- The next slice can now write objects, write placement rows, build the manifest from real placement locators, and publish the active root/control state in the correct order.
- No measurement artifacts are included; this checkpoint makes no benchmark or recall claim.
