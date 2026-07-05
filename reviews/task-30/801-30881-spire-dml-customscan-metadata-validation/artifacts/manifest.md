# Artifact Manifest: 30881 SPIRE DML CustomScan Metadata Validation

- head SHA: `7c107b1942d8dda885d4efa8b3bebbb11e0a7005`
- packet/topic: `30881-spire-dml-customscan-metadata-validation`
- timestamp: `2026-05-11T20:25:19-0700`
- storage format / rerank mode: not applicable; DML CustomScan metadata guard
  only
- isolated one-index-per-table or shared-table surfaces: not applicable for
  Rust unit validation; PG custom scan status tests use their own test tables

## Artifacts

### `cargo-test-custom-scan-lib.log`

- lane / fixture: focused Rust + pg_test lane for `custom_scan`
- command: `cargo test custom_scan --lib`
- key result lines:
  - `running 10 tests`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1668 filtered out`

### `cargo-fmt-check.log`

- lane / fixture: repository formatting check
- command: `cargo fmt --check`
- key result lines:
  - command exited 0
  - stable rustfmt emitted the known warnings about unstable
    `imports_granularity` and `group_imports`

### `git-diff-check.log`

- lane / fixture: whitespace check for the code commit
- command: `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs`
- key result lines:
  - command exited 0 with no whitespace errors
