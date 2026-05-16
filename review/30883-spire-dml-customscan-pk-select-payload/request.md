# Review Request: SPIRE DML CustomScan PK SELECT Payload Repair

## Scope

Code commit: `167c2befd55f26e6f7b95d0fba94b8c8f48256ac`

This packet repairs two issues uncovered while validating the live DML
PK SELECT CustomScan path before starting the UPDATE/DELETE plan-replacement
slice.

Changes:

- Stops mixing PostgreSQL `OidList` cells with string nodes in DML
  `CustomScan.custom_private`; DML plan-private metadata now uses a regular
  string-node list for mode, index OID, updated columns, and projected columns.
- Adds a shared plan-private `u32` decoder that still accepts the existing
  vector path's OID-list layout.
- Makes the PK SELECT primitive request the tuple columns required by the
  executor slot, falling back to the primitive projected columns only when the
  slot-column list is unavailable.
- Ensures the primary-key column is included in PK SELECT payload requests so
  PostgreSQL can evaluate the `WHERE pk = ...` qual against the virtual tuple.
- Adds unit coverage for the PK-column inclusion and tuple-slot column
  preference.
- Updates the Phase 11 task file with packet `30883` and the reviewer-confirmed
  UPDATE/DELETE plan-tree replacement direction from 30803 feedback seq 004.

UPDATE/DELETE execution remains disabled in this packet.

## Validation

- `cargo test custom_scan --lib`
  - `13 passed; 0 failed; 0 ignored; 1668 filtered out`
  - artifact: `artifacts/cargo-test-custom-scan-lib.log`
- `cargo test test_ec_spire_dml_frontdoor_pk_select_customscan_local_sql --lib`
  - `1 passed; 0 failed; 0 ignored; 1680 filtered out`
  - artifact: `artifacts/cargo-test-pk-select-customscan-local-sql.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check 167c2bef^ 167c2bef -- src/am/ec_spire/custom_scan.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the DML plan-private list repair avoids PostgreSQL mixed-list cell
   assertions while preserving vector CustomScan plan-private decoding.
2. Confirm PK SELECT payload requests use the executor slot's required columns
   and include the PK column needed for qual evaluation.
3. Confirm this is still a PK SELECT repair only; UPDATE/DELETE executor paths
   remain fail-closed.
