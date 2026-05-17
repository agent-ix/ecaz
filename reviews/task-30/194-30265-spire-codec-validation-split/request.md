# Review Request: SPIRE Codec Validation Split

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `8e16878b Split SPIRE codec validation from encoding`

## Scope

This packet covers an A8 pre-persistence architecture feedback slice: core
SPIRE partition-object codecs no longer use `encode()` as the validation
boundary for headers and assignment rows.

Changed files:

- `src/am/ec_spire/storage.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Split `SpirePartitionObjectHeader` into explicit format-version validation
  and a private post-validation encoder.
- Split `SpireLeafAssignmentRow` into wire-shape validation, checked encoded
  length calculation, and a private post-validation encoder.
- Updated leaf, delta, routing, V2 meta, and V2 segment validation to call
  validation helpers directly instead of encoding to prove shape.
- Updated leaf and delta object encoders to reuse their already validated row
  and header state when serializing child assignment rows.
- Made leaf and delta constructors validate header identity immediately, not
  only at later object-store write/encode time.
- Added a regression test for invalid PID/object-version rejection at object
  constructor boundaries.
- Updated Task 30 notes and the architecture feedback response with this
  checkpoint.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `176 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This is still an in-memory codec slice. It does not add relation-backed
persistence or buffer readers. The next related cleanup would be a shared object
reader contract so the current in-memory store and future buffer-cache reader
share the same validation boundary.
