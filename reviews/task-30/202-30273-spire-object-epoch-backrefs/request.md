# Review Request: SPIRE Object Epoch Backrefs

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `01e4b2c2 Add SPIRE object epoch backrefs`

## Scope

This packet covers the final pre-persistence architecture cleanup from the
first holistic SPIRE review: partition-object headers now carry a diagnostic
published-epoch back-reference.

Changed files:

- `src/am/ec_spire/storage.rs`
- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`
- `plan/design/spire-phase0-partition-object-storage.md`
- `spec/adr/ADR-049-spire-on-single-level-ivf-foundation.md`

## What Changed

- Added `published_epoch_backref` to the common partition-object header.
- Moved the existing V2 leaf metadata epoch back-reference into that common
  header field so V2 metadata and segment headers share the same value.
- Kept draft-created objects at `0`; local-store insertion stamps the durable
  encoded object with the write epoch.
- Updated object readers to reject zero backrefs and backrefs newer than the
  placement epoch, while still allowing later epoch manifests to reuse older
  immutable objects.
- Updated storage/build/update tests and the Phase 0, ADR-049, task, and
  architecture-response notes.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `181 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

The epoch back-reference is diagnostic, not the compatibility authority. Epoch
manifests still decide which object versions are valid for a query, and later
epochs may reference objects first published by older epochs.
