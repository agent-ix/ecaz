# Review Request: SPIRE Placement State Constructors

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `4bb30d3c Add explicit SPIRE placement state constructors`

## Scope

This packet covers a small architecture follow-up cleanup: local placement
construction now makes placement state explicit instead of requiring callers to
mutate `state` after construction.

Changed files:

- `src/am/ec_spire/meta.rs`
- `src/am/ec_spire/storage.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Added explicit local single-store placement constructors for:
  `Available`, `Stale`, `Unavailable`, and `Skipped`.
- Kept the existing `local_single_store` helper as an available-placement
  compatibility alias.
- Updated local object-store writes to call the available-state constructor
  directly.
- Added a test covering all explicit state constructors and their default local
  node/store IDs.
- Updated the Task 30 notes and architecture feedback response.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `178 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This does not change placement encoding. It only removes the need for callers to
construct an available placement and mutate state for degraded or failure-path
tests and future relation-backed placement writers.
