# Artifact Manifest: SPIRE READ Schema Drift Guard

- head SHA: `9d3c7b9cab65162b0ac2a4437d0d116b75c2ed4e`
- packet/topic: `763-c1-spire-read-schema-drift-guard`
- lane: Phase 12c.4 READ schema drift
- fixture: `test_ec_spire_customscan_read_schema_drift_variants_sql`
- storage format: `rabitq`
- rerank mode: not applicable
- timestamp: `2026-05-15T02:59:56Z`
- isolated one-index-per-table vs shared-table surface: isolated fixture tables
  per drift variant

## Artifacts

### `cargo-fmt-check.log`

- Command:
  `cargo fmt --check`
- Result: passed.
- Key result lines: only the existing stable-channel rustfmt warnings for
  unstable `imports_granularity` / `group_imports` settings.

### `cargo-test-read-schema-drift-no-run.log`

- Command:
  `cargo test --no-default-features --features "pg18 pg_test" test_ec_spire_customscan_read_schema_drift_variants_sql --no-run`
- Result: passed.
- Key result line:
  `Finished test profile [unoptimized + debuginfo]`

### `cargo-pgrx-test-read-schema-drift.log`

- Command:
  `cargo pgrx test pg18 test_ec_spire_customscan_read_schema_drift_variants_sql`
- Result: blocked by local harness loader before the test body ran.
- Key result line:
  `undefined symbol: pg_re_throw`

### `git-diff-check.log`

- Command:
  `git diff --check`
- Result: passed.
