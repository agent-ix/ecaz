# Artifact Manifest: 30887 SPIRE DML CustomScan DELETE Executor

- head SHA: `715b35dfea33a8e0492c067e0c54a34a2c23e1f8`
- packet/topic: `30887-spire-dml-customscan-delete-executor`
- timestamp: `2026-05-11T22:07:34-0700`
- storage format / rerank mode: not applicable; DML CustomScan DELETE executor
  wiring only
- isolated one-index-per-table or shared-table surfaces: focused PG18 pg_test
  fixtures create their own tables and one `ec_spire` index per table; Rust
  unit validation has no table surface

## Artifacts

### `cargo-test-dml-plan-tree-replace-delete.log`

- lane / fixture: focused PG18 DML plan-tree replacement and DELETE executor
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
- command: `git diff --check 715b35df^ 715b35df -- src/am/ec_spire/custom_scan.rs src/lib.rs`
- key result lines:
  - command exited 0 with no whitespace errors
