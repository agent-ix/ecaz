# Review Request: SPIRE Payload Helper Return

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `d99d2006 Drop SPIRE payload dimension return`

## Scope

This packet covers a small architecture-review cleanup: assignment payload
encoding no longer returns a source dimension value that all callers discarded.

Changed files:

- `src/am/ec_spire/quantizer.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Changed `encode_assignment_payload` from returning
  `(dimensions, gamma, encoded_payload)` to returning `(gamma, encoded_payload)`.
- Kept source vector shape and max-dimension validation in the payload helper.
- Updated assignment-input construction and quantizer/scorer tests to use the
  caller-owned source/query dimensions directly.
- Updated the Task 30 and architecture-response notes for the cleanup.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `181 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This keeps dimension ownership with the caller and avoids a misleading return
value. The payload helper still rejects empty, non-finite, and over-wide source
vectors before encoding.
