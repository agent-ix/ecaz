# Artifact Manifest: 30886 SPIRE DML CustomScan UPDATE Executor

- head SHA: `66e652290f32760323e48940cbbdddfc84cc0d52`
- packet/topic: `30886-spire-dml-customscan-update-executor`
- timestamp: `2026-05-11T22:02:24-0700`
- storage format / rerank mode: not applicable; DML CustomScan UPDATE executor
  wiring only
- isolated one-index-per-table or shared-table surfaces: focused PG18 pg_test
  fixtures create their own tables and one `ec_spire` index per table; Rust
  unit validation has no table surface

## Artifacts

### `cargo-test-dml-plan-tree-replace-update.log`

- lane / fixture: focused PG18 DML plan-tree replacement and UPDATE executor
  fixture
- command: `cargo test test_ec_spire_dml_plan_tree_replace_scaffold --lib`
- key result lines:
  - `test tests::pg_test_ec_spire_dml_plan_tree_replace_scaffold ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1682 filtered out`

### `cargo-test-custom-scan-lib.log`

- lane / fixture: focused Rust + pg_test lane for `custom_scan`
- command: `cargo test custom_scan --lib`
- key result lines:
  - `running 14 tests`
  - `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1669 filtered out`

### `cargo-fmt-check.log`

- lane / fixture: repository formatting check
- command: `cargo fmt --check`
- key result lines:
  - command exited 0
  - stable rustfmt emitted the known warnings about unstable
    `imports_granularity` and `group_imports`

### `git-diff-check.log`

- lane / fixture: whitespace check for the code commit
- command: `git diff --check 66e65229^ 66e65229 -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/dml_frontdoor.rs src/lib.rs`
- key result lines:
  - command exited 0 with no whitespace errors
