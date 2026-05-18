# Review Request: SPIRE Local Store Config Metadata

## Checkpoint

- Code commit: `b955491a`
  (`Add SPIRE local store config metadata`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 4 local store configuration metadata groundwork

## Summary

This checkpoint adds the first Phase 4 code slice below the local multi-store
placement design.

`src/am/ec_spire/meta.rs` now has:

- `SpireLocalStoreState`
- `SpireLocalStoreDescriptor`
- `SpireLocalStoreConfig`
- an embedded single-store config constructor for the current default shape
- encode/decode support for a versioned active store generation
- active-store placement validation for `node_id`, `local_store_id`,
  `store_relid`, and unavailable-store state
- a descriptor-backed `SpirePlacementEntry::local_store_available` constructor

The codec intentionally allows repeated `tablespace_oid` values. That lets a
developer create multiple logical local stores on the same tablespace or NVMe
for same-device baseline tests, while later diagnostics can still report the
actual repeated tablespace identity and avoid presenting it as physical
multi-NVMe striping.

The Task 30 tracker now marks the metadata codec slice complete. Store
reloptions, root/control persistence wiring, auxiliary relation creation,
hash-routed object writes, store-grouped reads, and benchmark measurements
remain open.

## Files

- `src/am/ec_spire/meta.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

Please review whether this metadata shape is sufficient before relation-helper
code lands:

- whether `tablespace_oid = 0` should remain valid for default tablespace
  inheritance;
- whether repeated `tablespace_oid` values are clearly allowed only as
  configuration/diagnostic reality, not a performance claim;
- whether placement validation belongs in `SpireLocalStoreConfig` at this
  stage;
- whether the embedded single-store constructor preserves current placement
  semantics without forcing existing indexes through a migration path.

## Validation

- `cargo fmt --check`
- `cargo test embedded_single_store_config --lib`
- `cargo test local_store_config --lib`
- `git diff --check`
- `git diff --cached --check`

## Notes

No PostgreSQL or PG18 tests were run. This is a pure Rust metadata/codec slice
with focused unit coverage.
