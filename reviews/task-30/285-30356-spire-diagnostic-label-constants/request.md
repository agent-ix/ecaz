# 30356 SPIRE Diagnostic Label Constants

## Request

Review the small code hardening for stable assignment payload status labels.

## Scope

- Added named constants for SPIRE assignment payload status labels.
- Replaced inline literals in `assignment_payload_scannability`.
- Updated Task 30 status to mention that the documented labels now have named
  code constants.

## Behavior

No SQL output change is intended. The existing labels remain:

- `supported`
- `deferred_model_metadata`

The change keeps these operator-facing strings from being anonymous literals
inside the scannability match.

## Validation

- `cargo fmt`
- `cargo test --lib test_ec_spire_options_snapshot_sql --no-default-features --features pg18 -- --nocapture`
- `git diff --check`
