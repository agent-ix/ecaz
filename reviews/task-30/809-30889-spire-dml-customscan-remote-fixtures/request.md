# Review Request: SPIRE DML CustomScan Remote Fixtures

## Scope

Code commit: `e90be93fa47ee91d77928f02b1049d8e09d0ad0d`

This packet adds direct remote-placement coverage for the transparent DML
CustomScan paths after the UPDATE/DELETE executor slices.

Changes:

- Adds a transparent remote UPDATE fixture:
  - registers a loopback remote descriptor and remote placement row;
  - proves `EXPLAIN UPDATE ... WHERE id = ...` records
    `plan_tree_replaced_customscan`;
  - executes `UPDATE` through the coordinator and asserts `ROW_COUNT = 1`;
  - verifies the owning remote heap row changed.
- Adds a transparent remote DELETE fixture:
  - registers a loopback remote descriptor and remote placement row;
  - proves `EXPLAIN DELETE ... WHERE id = ...` records
    `plan_tree_replaced_customscan`;
  - executes `DELETE` through the coordinator and asserts `ROW_COUNT = 1`;
  - verifies the placement row is removed locally and the remote delete is
    prepared but not visible before transaction resolution.
- Updates the Phase 11 task file to mark the broad UPDATE/DELETE/PK SELECT
  transparent DML checklist in line with the accepted top-level CustomScan
  plan-tree replacement path.

No production code changes are included.

## Validation

- `cargo test dml_customscan --lib`
  - `2 passed; 0 failed; 0 ignored; 1683 filtered out`
  - artifact: `artifacts/cargo-test-dml-customscan-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check e90be93f^ e90be93f -- src/lib.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm these fixtures prove the transparent DML CustomScan path, not only
   the lower-level coordinator primitives.
2. Confirm the remote DELETE assertion correctly treats the remote prepared
   transaction as not visible before local transaction resolution.
3. Confirm the task-file checklist wording now matches the top-level
   CustomScan plan-tree replacement architecture.
