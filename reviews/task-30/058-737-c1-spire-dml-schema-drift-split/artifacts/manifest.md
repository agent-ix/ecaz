# Artifact Manifest: SPIRE DML Schema Drift Split

packet: `737-c1-spire-dml-schema-drift-split`
head_sha: `2f3c39388e3159b2897e1fb596e4c3d6d0483383`
date: 2026-05-14

No measurement artifacts are attached. This packet is a focused test
coverage checkpoint with static/compile validation only.

## Validation Commands

- `cargo fmt --check`
  - Result: passed, with existing stable-rustfmt warnings.
- `git diff --check -- src/tests/dml_frontdoor.rs src/tests/dml_schema_drift.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Result: passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_update_schema_drift_variants_sql --no-run`
  - Result: passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_delete_schema_drift_variants_sql --no-run`
  - Result: passed.
- `cargo pgrx test pg18 test_ec_spire_update_schema_drift_variants_sql`
  - Result: failed before test execution with `undefined symbol: pg_re_throw`.
