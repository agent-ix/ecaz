# Review Request: SPIRE Object Reader Trait

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `c702d23e Add SPIRE object reader trait`

## Scope

This packet covers a small architecture follow-up cleanup: object reads now have
a shared trait boundary that can be implemented by the current in-memory store
and the future buffer-cache reader.

Changed files:

- `src/am/ec_spire/storage.rs`
- `src/am/ec_spire/diagnostics.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Added `SpireObjectReader` with methods for:
  - object headers
  - routing objects
  - V1 leaf objects
  - V2 leaf objects
  - delta objects
- Implemented the trait for `SpireLocalObjectStore`.
- Updated snapshot diagnostics to consume `&impl SpireObjectReader` instead of
  the concrete local object store.
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

This only moves diagnostics to the trait boundary. Scan and update helpers still
take `SpireLocalObjectStore` directly; those should move to the trait boundary
when V2 leaf scan migration lands.
