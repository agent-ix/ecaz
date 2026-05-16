# Artifact Manifest: SPIRE Remote PK Read Isolation

- head SHA: `ada38951be9152c3e7fcb271954a43679648d3e4`
- packet/topic: `30937-spire-remote-pk-read-isolation`
- timestamp: `2026-05-12T22:30:22Z`
- isolated one-index-per-table or shared-table surfaces: isolated
  one-index-per-table loopback coordinator/remote fixture

## Artifacts

### `git-diff-check.log`

- lane: static whitespace validation
- fixture: code/docs diff for commit `ada38951`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "git diff --check HEAD^ HEAD" review/30937-spire-remote-pk-read-isolation/artifacts/git-diff-check.log`
- key result lines: command exited successfully with no diff whitespace
  diagnostics.

### `cargo-fmt-check.log`

- lane: Rust formatting validation
- fixture: code/docs diff for commit `ada38951`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "cargo fmt --check" review/30937-spire-remote-pk-read-isolation/artifacts/cargo-fmt-check.log`
- key result lines: command exited successfully; log contains only stable
  rustfmt warnings about ignored nightly-only import grouping settings.

### `cargo-pgrx-test-remote-pk-isolation.log`

- lane: PG18 focused pgrx isolation fixture
- fixture: `test_ec_spire_remote_pk_select_isolation_contract_sql`
- storage format: ecvector loopback remote/coordinator tables
- rerank mode: n/a
- command used: `script -q -c "cargo pgrx test pg18 test_ec_spire_remote_pk_select_isolation_contract_sql" review/30937-spire-remote-pk-read-isolation/artifacts/cargo-pgrx-test-remote-pk-isolation.log`
- key result lines:
  - `test tests::pg_test_ec_spire_remote_pk_select_isolation_contract_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1688 filtered out`
