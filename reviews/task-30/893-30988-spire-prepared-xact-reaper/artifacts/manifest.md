# Artifact Manifest: 30988 SPIRE Prepared-Xact Reaper

- head SHA: `35240a79` (`Add SPIRE prepared xact reaper`)
- packet/topic: `30988-spire-prepared-xact-reaper`
- lane: Phase 12a.1 orphaned prepared-xact reaper
- fixture: PG18 loopback lost-ack prepared transaction fixture plus focused unit checks
- storage format: SPIRE default fixture index
- rerank mode: not applicable
- isolated one-index-per-table vs shared-table surfaces: isolated one-index-per-table test tables

## Artifacts

### `cargo-test-prepared-gid-parser.log`

- command: `cargo test prepared_transaction_gid_parser_extracts_reaper_identity --lib`
- timestamp: `2026-05-13 09:03:11-07:00`
- key lines:
  - `test am::ec_spire::production_executor_state_tests::prepared_transaction_gid_parser_extracts_reaper_identity ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1710 filtered out`

### `cargo-test-prepared-intent-state.log`

- command: `cargo test prepared_transaction_intent_state_validator_matches_catalog_contract --lib`
- timestamp: `2026-05-13 09:03:20-07:00`
- key lines:
  - `test am::ec_spire::production_executor_state_tests::prepared_transaction_intent_state_validator_matches_catalog_contract ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1710 filtered out`

### `cargo-pgrx-test-reaper-lost-ack-pg18.log`

- command: `cargo pgrx test pg18 test_ec_spire_reaper_resolves_lost_prepare_ack_fixture`
- timestamp: `2026-05-13 09:06:10-07:00`
- key lines:
  - `test tests::pg_test_ec_spire_reaper_resolves_lost_prepare_ack_fixture ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1710 filtered out; finished in 30.99s`

### `cargo-fmt-check.log`

- command: `cargo fmt --check`
- timestamp: `2026-05-13 09:06:15-07:00`
- key lines:
  - `COMMAND_EXIT_CODE="0"`

### `git-diff-check.log`

- command: `git diff --check`
- timestamp: `2026-05-13 09:06:19-07:00`
- key lines:
  - `COMMAND_EXIT_CODE="0"`
