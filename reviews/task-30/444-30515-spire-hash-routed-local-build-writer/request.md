# Review Request: SPIRE Hash-Routed Local Build Writer

## Checkpoint

- Code commit: `2634a593`
  (`Add SPIRE hash routed local build writer`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 local hash-routed build writer surface

## Summary

This checkpoint adds a testable object-store set that writes partition objects
by `SpireLocalStoreConfig::store_for_pid(pid)`.

The change:

- adds `SpireLocalObjectStoreSet`, which owns a local store config and one
  in-memory object store per descriptor;
- routes routing, leaf V2, and delta writes to the store selected by the stable
  PID hash rule;
- implements `SpireObjectReader` for the store set by validating placement
  entries against the active store config before reading from the selected
  store;
- generalizes the build object-store trait so single-level and recursive build
  draft code can write through either a single store, a store set, or the
  relation-backed store;
- adds coverage proving partitioned build root and leaf placements carry the
  expected hashed local store IDs and store relids;
- records the completed local writer surface in the Task 30 tracker.

This is deliberately not the final relation-backed multi-store writer. The
current executable PostgreSQL build path still blocks `local_store_count > 1`
until auxiliary store relations are created and opened.

## Files

- `src/am/ec_spire/storage.rs`
- `src/am/ec_spire/build.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

Please review:

- whether `SpireLocalObjectStoreSet` is the right shape for exercising
  multi-store placement before DDL-backed relation stores exist;
- whether routing reads through `SpireLocalStoreConfig::validate_placement`
  belongs at this store-set layer;
- whether the generalized build writer trait should stay in `build.rs` or move
  to `storage.rs` before relation-backed store sets land;
- whether keeping the main relation-backed hash-routed write item open is the
  right tracker state.

## Validation

- `cargo fmt --check`
- `cargo test hash_routes_object_writes --lib`
- `cargo test partitioned_single_level_draft --lib`
- `cargo test local_object_store --lib`
- `cargo test local_store_config --lib`
- `git diff --check`
- `git diff --cached --check`

## Notes

No PostgreSQL integration tests were run. This slice changes Rust build draft
and in-memory store behavior; relation-backed multi-store execution remains
guarded by the existing `local_store_count > 1` ambuild error.
