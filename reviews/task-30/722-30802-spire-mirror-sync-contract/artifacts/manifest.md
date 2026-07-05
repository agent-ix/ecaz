# Artifact Manifest: SPIRE Mirror Sync Contract

- head SHA: historical Shape-A checkpoint
- packet/topic: `30802-spire-mirror-sync-contract`
- lane: superseded mirror-sync / remote row-materialization contract
- fixture: focused PG18 pgrx contract tests
- storage format: not recorded
- rerank mode: not recorded
- timestamp: `2026-05-10T20:36:44-07:00`
- isolated one-index-per-table vs shared-table surface: historical contract
  logs; not used as current production-readiness evidence

## Artifacts

### `pg18-remote-search-final-contract.log`

- Command:
  `cargo pgrx test pg18 test_ec_spire_remote_search_final_contract`
- Result: passed.
- Key result line:
  `test tests::pg_test_ec_spire_remote_search_final_contract ... ok`

### `pg18-remote-phase7-policy-contracts.log`

- Command:
  `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- Result: passed.
- Key result line:
  `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`
