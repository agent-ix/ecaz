# Artifact Manifest: 30723 SPIRE Production Executor Dry State

Head SHA: `ab64003b9ecf0571ccb9ddaee24b861e527b6e49`
Packet/topic: `review/30723-spire-production-executor-dry-state`

## Artifacts

### `git-diff-check.log`

- Lane: code diff validation
- Fixture: committed diff from `38c807ec` to `ab64003b`
- Storage format: not applicable
- Rerank mode: not applicable
- Command:
  `git diff 38c807ec ab64003b --check`
- Timestamp: 2026-05-10 01:59:34 -0700
- Isolated one-index-per-table or shared-table surface: not applicable
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Lane: formatting
- Fixture: repository source tree at `ab64003b`
- Storage format: not applicable
- Rerank mode: not applicable
- Command:
  `cargo fmt --check`
- Timestamp: 2026-05-10 01:59:40 -0700
- Isolated one-index-per-table or shared-table surface: not applicable
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`

### `cargo-check-pg18.log`

- Lane: PG18 compile
- Fixture: repository source tree at `ab64003b`
- Storage format: not applicable
- Rerank mode: not applicable
- Command:
  `cargo check --no-default-features --features pg18`
- Timestamp: 2026-05-10 01:59:47 -0700
- Isolated one-index-per-table or shared-table surface: not applicable
- Key result lines:
  - `Finished dev profile ... target(s) in 0.12s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-test-production-executor-state.log`

- Lane: focused Rust and PG18 dry-state coverage
- Fixture: production executor C0 state and dry SQL summary
- Storage format: not applicable
- Rerank mode: not applicable
- Command:
  `cargo test production_executor_state --lib`
- Timestamp: 2026-05-10 02:00:06 -0700
- Isolated one-index-per-table or shared-table surface: shared-table PG18 test
  surface
- Key result lines:
  - `production_executor_state_keeps_admitted_dispatches_dry ... ok`
  - `production_executor_state_preserves_pre_dispatch_blocker ... ok`
  - `test tests::pg_test_ec_spire_production_executor_state_summary_is_dry ... ok`
  - `3 passed; 0 failed`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-pgrx-pg18-operator-contracts.log`

- Lane: focused PG18 operator contract coverage
- Fixture: remote operator entrypoint contract
- Storage format: not applicable
- Rerank mode: not applicable
- Command:
  `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- Timestamp: 2026-05-10 02:02:10 -0700
- Isolated one-index-per-table or shared-table surface: shared-table PG18 test
  surface
- Key result lines:
  - `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`
  - `1 passed; 0 failed`
  - `COMMAND_EXIT_CODE="0"`
