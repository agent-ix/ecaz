# Review Request: SPIRE Object Store Local Store ID Surface

## Checkpoint

- Code commit: `7beb5d05`
  (`Carry SPIRE local store ids through object stores`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 object-store placement surface

## Summary

This checkpoint removes the object-store layer's hard dependency on
`local_store_id = 0` when creating and validating SPIRE placement entries.

The change:

- lets `SpireLocalObjectStore` carry a configured `local_store_id`;
- adds a descriptor-backed local object-store constructor for tests and future
  hash-routed writers;
- rejects writes through descriptors whose store state is not `Available`;
- updates relation-backed object-store placement creation and validation to use
  the wrapper's configured local store ID;
- keeps `SpireRelationObjectStore::for_index_relation` defaulting to store `0`
  for today's embedded single-store path;
- adds a placement constructor that accepts explicit `(local_store_id,
  store_relid)` values;
- records the completed surface in the Task 30 tracker.

This does not yet create auxiliary store relations or route build writes by
hash. It prepares the object codec and placement layer so the next writer slice
can select a store descriptor instead of changing placement entries after the
fact.

## Files

- `src/am/ec_spire/storage.rs`
- `src/am/ec_spire/meta.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

Please review:

- whether `SpireLocalObjectStore::for_store_descriptor` is the right test and
  future-writer entry point;
- whether relation-backed store validation should remain configured by wrapper
  state rather than active config lookup at this layer;
- whether keeping `for_index_relation` as the embedded store-0 constructor is
  clear enough until dedicated store-relation open helpers land;
- whether the explicit placement constructor belongs in `meta.rs` or should
  instead require a full descriptor at every call site.

## Validation

- `cargo fmt --check`
- `cargo test local_object_store --lib`
- `cargo test local_store_config --lib`
- `git diff --check`
- `git diff --cached --check`

## Notes

No PostgreSQL integration tests were run for this slice. The executable
relation-backed path still opens the index relation as the embedded single
store.
