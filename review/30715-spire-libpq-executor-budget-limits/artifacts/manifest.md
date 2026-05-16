# Artifact Manifest

head_sha: a27bc8d613612a0166dcc5e33380cf4b1199fafe
packet: 30715-spire-libpq-executor-budget-limits
lane: Phase 11 Stage C production libpq coordinator
timestamp: 2026-05-10T00:08:00-07:00

## cargo-check-pg18.log

- Command: `cargo check --no-default-features --features pg18`
- Fixture: PG18 compile check.
- Storage format / rerank mode: not applicable.
- Isolated/shared surface: code compile only.
- Key result:
  - `Finished dev profile ... target(s) in 0.12s`
  - command exit code 0.

## cargo-pgrx-pg18-libpq-budget-limits.log

- Command: `cargo pgrx test pg18 test_ec_spire_libpq_executor_budget_limits`
- Fixture: PG18 budget contract probe plus SQL-visible local budget summary.
- Storage format / rerank mode: default local SPIRE fixture; no remote scoring
  benchmark.
- Isolated/shared surface: diagnostic SQL surfaces and synthetic executor
  dispatch rows; no shared-table performance claim.
- Key result:
  - `test tests::pg_test_ec_spire_libpq_executor_budget_limits ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1521 filtered out`

## cargo-pgrx-pg18-receive-contract.log

- Command: `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`
- Fixture: PG18 libpq receive/parameter/result contract coverage.
- Storage format / rerank mode: contract-only; no benchmark.
- Isolated/shared surface: diagnostic contract SQL surfaces.
- Key result:
  - `test tests::pg_test_ec_spire_remote_search_receive_contract ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1521 filtered out`

## cargo-pgrx-pg18-phase7-policy-contracts.log

- Command: `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- Fixture: PG18 operator-entrypoint and remote policy contract coverage.
- Storage format / rerank mode: contract-only; no benchmark.
- Isolated/shared surface: diagnostic contract SQL surfaces.
- Key result:
  - `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1521 filtered out`

## cargo-pgrx-pg18-libpq-loopback.log

- Command: `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- Fixture: PG18 loopback remote executor with compact and heap receive.
- Storage format / rerank mode: remote loopback index uses
  `storage_format = 'rabitq'`; rerank mode not a benchmark variable.
- Isolated/shared surface: loopback diagnostic executor surface.
- Key result:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1521 filtered out`

## git-diff-check.log

- Command: `git diff --check`
- Fixture: whitespace/check-only validation.
- Storage format / rerank mode: not applicable.
- Isolated/shared surface: not applicable.
- Key result:
  - command exit code 0.
