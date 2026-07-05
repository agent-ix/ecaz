# Review Request: SPIRE DML CustomScan Column Metadata

## Scope

Code commit: `ce8676e7af4e3d5e60eff46ae40ff3d2c3b48cab`

This packet extends the DML `EcSpireDistributedScan` plan/executor handoff with
the operation-specific column metadata needed by the upcoming transparent
UPDATE/DELETE executor slices.

Changes:

- Serializes DML primitive column metadata into `CustomScan.custom_private`
  after the existing mode and index OID entries.
- Stores UPDATE `updated_columns` and PK SELECT `projected_columns` as JSON
  string nodes so column names are not lossy or comma-delimited.
- Decodes that metadata during `BeginCustomScan` for all DML plan modes.
- Adds unit coverage for metadata round-trip and fail-closed empty-column
  handling.
- Updates the Phase 11 task file with packet `30880`.

This packet does not enable UPDATE/DELETE path generation or remove the current
UPDATE/DELETE executor guard. The live DML CustomScan path remains PK SELECT.

## Validation

- `cargo test custom_scan --lib`
  - `9 passed; 0 failed; 0 ignored; 1668 filtered out`
  - artifact: `artifacts/cargo-test-custom-scan-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm using JSON string nodes in `custom_private` is an acceptable
   serialization boundary for DML column metadata.
2. Confirm the additional plan-private entries preserve the existing
   mode/index OID layout consumed by vector and PK SELECT CustomScan paths.
3. Confirm this remains a metadata-only slice and does not accidentally enable
   UPDATE/DELETE execution.
