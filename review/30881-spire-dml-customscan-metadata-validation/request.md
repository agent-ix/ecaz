# Review Request: SPIRE DML CustomScan Metadata Validation

## Scope

Code commit: `7c107b1942d8dda885d4efa8b3bebbb11e0a7005`

This packet adds a mode-specific fail-closed guard for the DML column metadata
carried by `EcSpireDistributedScan` plan-private state.

Changes:

- Validates DML column metadata during `BeginCustomScan` after decoding the
  plan-private JSON string nodes.
- Enforces the operation-specific shape:
  - PK SELECT requires projected columns and no updated columns.
  - UPDATE requires updated columns and no projected columns.
  - DELETE carries no column payload metadata.
- Adds unit coverage for valid and invalid mode/metadata combinations.
- Updates the Phase 11 task file with packet `30881`.

This packet still does not enable UPDATE/DELETE path generation or remove the
existing UPDATE/DELETE executor guard.

## Validation

- `cargo test custom_scan --lib`
  - `10 passed; 0 failed; 0 ignored; 1668 filtered out`
  - artifact: `artifacts/cargo-test-custom-scan-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the DML metadata guard matches the primitive-plan contract from
   `dml_frontdoor.rs`.
2. Confirm the guard is placed at the right executor boundary (`BeginCustomScan`)
   before any DML primitive invocation.
3. Confirm this remains non-behavioral for live UPDATE/DELETE routing because
   those path generators are still disabled.
