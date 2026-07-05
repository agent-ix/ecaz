# Artifact Manifest

- head SHA: `0d96d6a6c4c381469ac4c1972b7d996359a62d71`
- packet/topic: `30652-spire-drop-index-catalog-cleanup`
- timestamp: `2026-05-09T05:59:32Z`
- isolated one-index-per-table or shared-table surfaces: isolated one-index-per-table DROP INDEX fixture

## Validation Runs

### drop-index event cleanup

- lane: PG18 focused pgrx test
- fixture: one real `ec_spire` index plus synthetic remote descriptor/manifest rows keyed to that index OID
- storage format: SPIRE/ecvector SQL fixture
- rerank mode: not applicable
- command: `cargo pgrx test pg18 test_ec_spire_remote_catalog_drop_index_event_cleanup`
- key result lines:
  - `test tests::pg_test_ec_spire_remote_catalog_drop_index_event_cleanup ... ok`
  - `test result: ok. 1 passed; 0 failed`

### phase7 policy contract

- lane: PG18 focused pgrx test
- fixture: contract-only
- storage format: not applicable
- rerank mode: not applicable
- command: `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- key result lines:
  - `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`
  - `test result: ok. 1 passed; 0 failed`

### whitespace

- command: `git diff --check`
- key result lines:
  - no output
