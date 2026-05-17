# Review Request: SPIRE Object Byte Diagnostics

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `0de3bb9b Split SPIRE diagnostics object bytes by kind`

## Scope

This packet covers a small architecture follow-up cleanup: snapshot diagnostics
now break available object bytes down by object kind.

Changed files:

- `src/am/ec_spire/diagnostics.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Kept the existing aggregate `available_object_bytes` diagnostic.
- Added per-kind byte buckets for:
  - routing objects, covering root/internal routing objects
  - leaf objects
  - delta objects
- Updated diagnostics tests to assert the aggregate matches the sum of kind
  buckets for available placements.
- Updated the Task 30 notes and architecture feedback response.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `181 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This keeps graph/future object kinds out of the routing bucket. When graph
objects land, they should add a separate byte counter rather than broadening the
routing bucket.
