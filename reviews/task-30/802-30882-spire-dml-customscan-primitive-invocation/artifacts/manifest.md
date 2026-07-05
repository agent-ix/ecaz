# Artifact Manifest: 30882 SPIRE DML CustomScan Primitive Invocation

- head SHA: `eada9ab73464bdc4967907774717c19d4896f0fc`
- packet/topic: `30882-spire-dml-customscan-primitive-invocation`
- timestamp: `2026-05-11T20:31:47-0700`
- storage format / rerank mode: not applicable; DML CustomScan executor
  invocation boundary only
- isolated one-index-per-table or shared-table surfaces: not applicable for
  Rust unit validation; PG custom scan status tests use their own test tables

## Artifacts

### `cargo-test-custom-scan-lib.log`

- lane / fixture: focused Rust + pg_test lane for `custom_scan`
- command: `cargo test custom_scan --lib`
- key result lines:
  - `running 12 tests`
  - `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1668 filtered out`

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
