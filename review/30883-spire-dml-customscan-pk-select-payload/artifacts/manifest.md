# Artifact Manifest: 30883 SPIRE DML CustomScan PK SELECT Payload Repair

- head SHA: `167c2befd55f26e6f7b95d0fba94b8c8f48256ac`
- packet/topic: `30883-spire-dml-customscan-pk-select-payload`
- timestamp: `2026-05-11T21:23:35-0700`
- storage format / rerank mode: not applicable; DML CustomScan PK SELECT
  plan-private and tuple-payload repair only
- isolated one-index-per-table or shared-table surfaces: focused PG18 pg_test
  fixtures create their own tables; Rust unit validation has no table surface

## Artifacts

### `cargo-test-custom-scan-lib.log`

- lane / fixture: focused Rust + pg_test lane for `custom_scan`
- command: `cargo test custom_scan --lib`
- key result lines:
  - `running 13 tests`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1668 filtered out`

### `cargo-test-pk-select-customscan-local-sql.log`

- lane / fixture: focused PG18 DML PK SELECT CustomScan local placement fixture
- command: `cargo test test_ec_spire_dml_frontdoor_pk_select_customscan_local_sql --lib`
- key result lines:
  - `test tests::pg_test_ec_spire_dml_frontdoor_pk_select_customscan_local_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1680 filtered out`

### `cargo-fmt-check.log`

- lane / fixture: repository formatting check
- command: `cargo fmt --check`
- key result lines:
  - command exited 0
  - stable rustfmt emitted the known warnings about unstable
    `imports_granularity` and `group_imports`

### `git-diff-check.log`

- lane / fixture: whitespace check for the code commit
- command: `git diff --check 167c2bef^ 167c2bef -- src/am/ec_spire/custom_scan.rs`
- key result lines:
  - command exited 0 with no whitespace errors
