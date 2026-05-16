# Artifact Manifest: SPIRE Insert Descriptor Race

- head SHA: `121cc46bae02850fcd9c41d6c4452ea88ae41070`
- packet/topic: `30938-spire-insert-descriptor-race`
- timestamp: `2026-05-12T22:52:54Z`
- isolated one-index-per-table or shared-table surfaces: isolated
  one-index-per-table loopback coordinator/remote fixture

## Artifacts

### `git-diff-check.log`

- lane: static whitespace validation
- fixture: code/docs diff for commit `121cc46b`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "git diff --check HEAD^ HEAD" review/30938-spire-insert-descriptor-race/artifacts/git-diff-check.log`
- key result lines: command exited successfully with no diff whitespace
  diagnostics.

### `cargo-fmt-check.log`

- lane: Rust formatting validation
- fixture: code/docs diff for commit `121cc46b`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "cargo fmt --check" review/30938-spire-insert-descriptor-race/artifacts/cargo-fmt-check.log`
- key result lines: command exited successfully; log contains only stable
  rustfmt warnings about ignored nightly-only import grouping settings.

### `cargo-pgrx-test-insert-descriptor-race.log`

- lane: PG18 focused pgrx coordinator INSERT race fixture
- fixture: `test_ec_spire_insert_descriptor_race_sql`
- storage format: ecvector loopback remote/coordinator tables
- rerank mode: n/a
- command used: `script -q -c "cargo pgrx test pg18 test_ec_spire_insert_descriptor_race_sql" review/30938-spire-insert-descriptor-race/artifacts/cargo-pgrx-test-insert-descriptor-race.log`
- key result lines:
  - `test tests::pg_test_ec_spire_insert_descriptor_race_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1689 filtered out`
