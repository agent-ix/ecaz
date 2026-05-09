# Artifact Manifest

- head SHA: `6d0043047b4228bb08469741a4bcad3acb0c16bf`
- packet/topic: `30651-spire-remote-heap-libpq-candidates`
- timestamp: `2026-05-09T05:37:01Z`
- isolated one-index-per-table or shared-table surfaces: isolated one-index-per-table loopback fixture

## Validation Runs

### search libpq remote heap candidates loopback

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
