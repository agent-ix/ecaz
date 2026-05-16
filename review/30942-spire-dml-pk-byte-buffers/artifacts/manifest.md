# Artifact Manifest: SPIRE DML PK Byte Buffers

- head SHA: `7c5eac3032f4b1be4d0235bd110d8c45f740e9f7`
- packet/topic: `30942-spire-dml-pk-byte-buffers`
- timestamp: `2026-05-12T23:45:52Z`
- isolated one-index-per-table or shared-table surfaces: isolated local DML
  frontdoor and CustomScan executor fixtures

## Artifacts

### `git-diff-check.log`

- lane: static whitespace validation
- fixture: code/tracker diff for commit `7c5eac30`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "git diff --check HEAD^ HEAD" review/30942-spire-dml-pk-byte-buffers/artifacts/git-diff-check.log`
- key result lines: command exited successfully with no diff whitespace
  diagnostics.

### `cargo-fmt-check.log`

- lane: Rust formatting validation
- fixture: code/tracker diff for commit `7c5eac30`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "cargo fmt --check" review/30942-spire-dml-pk-byte-buffers/artifacts/cargo-fmt-check.log`
- key result lines: command exited successfully; log contains only stable
  rustfmt warnings about ignored nightly-only import grouping settings.

### `cargo-pgrx-test-pk-value-bytes.log`

- lane: PG18 focused pgrx bigint byte parity fixture
- fixture: `test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "cargo pgrx test pg18 test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send" review/30942-spire-dml-pk-byte-buffers/artifacts/cargo-pgrx-test-pk-value-bytes.log`
- key result lines:
  - `test tests::pg_test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1688 filtered out`

### `cargo-pgrx-test-primitive-plan.log`

- lane: PG18 focused pgrx primitive plan and runtime parameter fixture
- fixture: `test_ec_spire_dml_frontdoor_primitive_plan_from_decision`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "cargo pgrx test pg18 test_ec_spire_dml_frontdoor_primitive_plan_from_decision" review/30942-spire-dml-pk-byte-buffers/artifacts/cargo-pgrx-test-primitive-plan.log`
- key result lines:
  - `test tests::pg_test_ec_spire_dml_frontdoor_primitive_plan_from_decision ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1688 filtered out`
