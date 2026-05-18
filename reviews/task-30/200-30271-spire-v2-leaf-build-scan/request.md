# Review Request: SPIRE V2 Leaf Build Scan

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `6eb8a47f Write and scan SPIRE V2 leaf objects`

## Scope

This packet covers the next Task 30 architecture follow-up: build-produced base
leaves now use the segmented column-major `LeafPartitionObjectV2` storage path,
and scan/update/diagnostics helpers can consume those V2 placements.

Changed files:

- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/scan.rs`
- `src/am/ec_spire/update.rs`
- `src/am/ec_spire/diagnostics.rs`
- `src/am/ec_spire/storage.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Switched single-level and partitioned build drafts from V1 leaf-object writes
  to `insert_leaf_object_v2_from_rows`.
- Extended object-header dispatch so a placement pointing at a V2 metadata
  tuple resolves the logical leaf header and validates total object bytes.
- Added a V2-to-owned-row compatibility bridge for scan and delta-update paths,
  keeping V1 compatibility reads intact.
- Updated diagnostics to count leaf assignments from the validated header, so
  V2 leaf placements do not require a V1 object read.
- Updated storage/build/update tests and the Task 30 architecture notes.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `181 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This closes the A1 storage-shape blocker for in-memory build outputs: base
leaves are no longer single page-bounded row-contiguous tuples. A2 remains open:
scan can read V2 leaves, but it still reconstructs owned rows before scoring.
The next hot-path slice should drive candidate scoring directly from borrowed
V2 column views.
