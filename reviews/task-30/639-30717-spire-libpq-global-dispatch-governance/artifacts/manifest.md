# Artifact Manifest

head_sha: 6e0186a826a324e68ec1034f662fc7cb89a72a2d
packet: 30717-spire-libpq-global-dispatch-governance
lane: Phase 11 Stage C production libpq coordinator
timestamp: 2026-05-10T07:53:45Z

## cargo-check-pg18.log

- Command: `cargo check --no-default-features --features pg18`
- Fixture: PG18 compile check.
- Storage format / rerank mode: not applicable.
- Isolated/shared surface: code compile only.
- Key result:
  - `Finished dev profile ... target(s) in 0.13s`
  - command exit code 0.

## cargo-pgrx-pg18-global-governance-overload.log

- Command: `cargo pgrx test pg18 test_ec_spire_libpq_executor_global_governance_overload`
- Fixture: PG18 coordinator fixture with a separate PostgreSQL backend holding
  the global remote-search governance advisory slot.
- Storage format / rerank mode: local SPIRE coordinator fixture; remote scoring
  is intentionally blocked before conninfo lookup, so this is not a scoring or
  performance benchmark.
- Isolated/shared surface: shared advisory-lock admission across backend
  sessions.
- Key result:
  - `test tests::pg_test_ec_spire_libpq_executor_global_governance_overload ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1523 filtered out`

## cargo-pgrx-pg18-budget-limits.log

- Command: `cargo pgrx test pg18 test_ec_spire_libpq_executor_budget_limits`
- Fixture: PG18 budget contract probe plus SQL-visible budget summary.
- Storage format / rerank mode: default local SPIRE fixture; no remote scoring
  benchmark.
- Isolated/shared surface: diagnostic SQL surfaces and synthetic executor
  dispatch rows; no shared-table performance claim.
- Key result:
  - `test tests::pg_test_ec_spire_libpq_executor_budget_limits ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1523 filtered out`

## cargo-pgrx-pg18-libpq-loopback.log

- Command: `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- Fixture: PG18 ready loopback remote executor with compact receive, remote
  heap receive, coordinator summary, and identity-cache summary assertions.
- Storage format / rerank mode: remote loopback index uses
  `storage_format = 'rabitq'`; rerank mode is not a benchmark variable.
- Isolated/shared surface: loopback diagnostic executor surface.
- Key result:
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1523 filtered out`

## git-diff-check.log

- Command: `git diff --check`
- Fixture: whitespace/check-only validation.
- Storage format / rerank mode: not applicable.
- Isolated/shared surface: not applicable.
- Key result:
  - command exit code 0.
