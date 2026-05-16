# Artifact Manifest: 30889 SPIRE DML CustomScan Remote Fixtures

- head SHA: `e90be93fa47ee91d77928f02b1049d8e09d0ad0d`
- packet/topic: `30889-spire-dml-customscan-remote-fixtures`
- timestamp: `2026-05-11T22:23:01-0700`
- storage format / rerank mode: not applicable; transparent DML CustomScan
  remote-placement fixtures only
- isolated one-index-per-table or shared-table surfaces: focused PG18 pg_test
  fixtures create their own coordinator and loopback remote tables, each with
  one `ec_spire` index

## Artifacts

### `cargo-test-dml-customscan-lib.log`

- lane / fixture: focused PG18 transparent DML CustomScan remote-placement
  fixtures
- command: `cargo test dml_customscan --lib`
- key result lines:
  - `test tests::pg_test_ec_spire_dml_customscan_remote_update_sql ... ok`
  - `test tests::pg_test_ec_spire_dml_customscan_remote_delete_sql ... ok`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1683 filtered out`

### `cargo-fmt-check.log`

- lane / fixture: repository formatting check
- command: `cargo fmt --check`
- key result lines:
  - command exited 0
  - stable rustfmt emitted the known warnings about unstable
    `imports_granularity` and `group_imports`

### `git-diff-check.log`

- lane / fixture: whitespace check for the code commit
- command: `git diff --check e90be93f^ e90be93f -- src/lib.rs`
- key result lines:
  - command exited 0 with no whitespace errors
