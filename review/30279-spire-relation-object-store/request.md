# Review Request: SPIRE Relation Object Store

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `5f85754c Add SPIRE relation object store`

## Summary

This checkpoint wraps the raw relation object tuple primitive in a SPIRE-owned
relation object store for routing objects. The store stamps the published epoch
back-reference, appends encoded object bytes to the `ec_spire` index relation,
emits a local single-store placement entry, and reads the routing object back
through that placement.

## Changed Files

- `src/am/ec_spire/storage.rs`
- `src/am/ec_spire/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## What Changed

- Added `SpireRelationObjectStore` over a PostgreSQL index relation.
- Added relation-store routing-object insertion that produces
  `SpirePlacementEntry::local_single_store_available` with the index relid as
  the Phase 1 store relid.
- Added relation-store routing-object reads that validate local placement
  shape, relation store relid, placement byte length, object PID/version, and
  epoch back-reference.
- Updated the pg round-trip helper/test so it uses the relation object store
  instead of directly appending raw bytes, and asserts the emitted placement
  store relid matches the index OID.
- Updated Task 30 status to distinguish raw object tuple persistence from the
  relation object store wrapper.

## Validation

- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    unstable `imports_granularity` and `group_imports` settings.
- `cargo test --lib test_ec_spire_relation_object_tuple_roundtrip --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1064 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `184 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

## Notes For Reviewer

- This checkpoint intentionally limits relation-store writes to routing objects.
  V2 leaf segment/meta writes and manifest persistence are the next storage
  surfaces before populated `ambuild` can publish a complete epoch.
- The relation store is scoped to the Phase 1 local single-store default; future
  multi-store routing should choose the relation before constructing the store.
- The untracked architecture-review feedback file
  `review/30219-spire-foundation-progress-status/feedback.md` remains local and
  was not staged or committed by this checkpoint.
