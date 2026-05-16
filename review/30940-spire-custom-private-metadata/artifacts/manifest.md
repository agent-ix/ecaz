# Artifact Manifest: SPIRE DML Custom Private Metadata

- head SHA: `6d8a544b4fced947c981f86905c1dca359cd4f70`
- packet/topic: `30940-spire-custom-private-metadata`
- timestamp: `2026-05-12T23:12:38Z`
- isolated one-index-per-table or shared-table surfaces: n/a; planner
  metadata/copyObject fixture only

## Artifacts

### `git-diff-check.log`

- lane: static whitespace validation
- fixture: code/tracker diff for commit `6d8a544b`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "git diff --check HEAD^ HEAD" review/30940-spire-custom-private-metadata/artifacts/git-diff-check.log`
- key result lines: command exited successfully with no diff whitespace
  diagnostics.

### `cargo-fmt-check.log`

- lane: Rust formatting validation
- fixture: code/tracker diff for commit `6d8a544b`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "cargo fmt --check" review/30940-spire-custom-private-metadata/artifacts/cargo-fmt-check.log`
- key result lines: command exited successfully; log contains only stable
  rustfmt warnings about ignored nightly-only import grouping settings.

### `cargo-pgrx-test-custom-private-copyobject.log`

- lane: PG18 focused pgrx copyObject fixture
- fixture: `test_ec_spire_custom_scan_dml_plan_private_copyobject_sql`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "cargo pgrx test pg18 test_ec_spire_custom_scan_dml_plan_private_copyobject_sql" review/30940-spire-custom-private-metadata/artifacts/cargo-pgrx-test-custom-private-copyobject.log`
- key result lines:
  - `test tests::pg_test_ec_spire_custom_scan_dml_plan_private_copyobject_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1687 filtered out`
