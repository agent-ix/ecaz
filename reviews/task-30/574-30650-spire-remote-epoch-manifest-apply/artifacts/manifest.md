# Artifact Manifest

- head SHA: `1d8499b96f6d224c8b14017770c9a26dd9125be8`
- packet/topic: `30650-spire-remote-epoch-manifest-apply`
- timestamp: `2026-05-09T05:29:25Z`
- isolated one-index-per-table or shared-table surfaces: isolated one-index-per-table loopback fixtures

## Validation Runs

### manifest libpq apply loopback

- lane: PG18 focused pgrx test
- fixture: coordinator and remote SPIRE indexes in one PG18 instance, with remote access through loopback libpq conninfo
- storage format: SPIRE/ecvector SQL fixture
- rerank mode: not applicable
- command: `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_libpq_executor_loopback`
- key result lines:
  - `test tests::pg_test_ec_spire_remote_epoch_manifest_libpq_executor_loopback ... ok`
  - `test result: ok. 1 passed; 0 failed`

### search libpq executor lookup signal loopback

- lane: PG18 focused pgrx test
- fixture: coordinator and remote SPIRE indexes in one PG18 instance, with remote access through loopback libpq conninfo
- storage format: SPIRE/ecvector SQL fixture
- rerank mode: strict executor candidate path
- command: `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- key result lines:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed`

### whitespace

- command: `git diff --check`
- key result lines:
  - no output
