# Artifact Manifest: SPIRE DML Relation Context Cache

- head SHA: `0886e3bd6ab873d3f10e8c4bcfc7f8c5e8b33275`
- packet/topic: `30941-spire-dml-relation-context-cache`
- timestamp: `2026-05-12T23:26:18Z`
- isolated one-index-per-table or shared-table surfaces: isolated local
  relation-context cache fixture

## Artifacts

### `git-diff-check.log`

- lane: static whitespace validation
- fixture: code/tracker diff for commit `0886e3bd`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "git diff --check HEAD^ HEAD" review/30941-spire-dml-relation-context-cache/artifacts/git-diff-check.log`
- key result lines: command exited successfully with no diff whitespace
  diagnostics.

### `cargo-fmt-check.log`

- lane: Rust formatting validation
- fixture: code/tracker diff for commit `0886e3bd`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "cargo fmt --check" review/30941-spire-dml-relation-context-cache/artifacts/cargo-fmt-check.log`
- key result lines: command exited successfully; log contains only stable
  rustfmt warnings about ignored nightly-only import grouping settings.

### `cargo-pgrx-test-dml-context-cache.log`

- lane: PG18 focused pgrx relcache invalidation fixture
- fixture: `test_ec_spire_dml_context_cache_invalidation_sql`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "cargo pgrx test pg18 test_ec_spire_dml_context_cache_invalidation_sql" review/30941-spire-dml-relation-context-cache/artifacts/cargo-pgrx-test-dml-context-cache.log`
- key result lines:
  - `test tests::pg_test_ec_spire_dml_context_cache_invalidation_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1688 filtered out`
