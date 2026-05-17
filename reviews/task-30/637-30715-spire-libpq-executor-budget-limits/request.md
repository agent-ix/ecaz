# 30715 — SPIRE libpq executor budget limits

Code commit: `a27bc8d613612a0166dcc5e33380cf4b1199fafe`

This packet opens the next Phase 11 Stage C slice: bounded remote-search libpq
dispatch before secret lookup or socket open.

## What Changed

- Added `plan/design/spire-libpq-executor-budget.md` and linked it from
  ADR-058.
- Added session GUCs:
  - `ec_spire.remote_search_max_nodes`
  - `ec_spire.remote_search_max_pids`
  - `ec_spire.remote_search_max_pids_per_node`
  - `ec_spire.remote_search_connect_timeout_ms`
  - `ec_spire.remote_search_statement_timeout_ms`
- Added dispatch admission gates. Ready rows over a nonzero cap now become
  `remote_executor_overload`, use `blocked_before_dispatch`, and surface
  `remote_executor_budget`.
- Added `ec_spire_remote_search_libpq_executor_budget_summary(...)`.
- Generalized secret summary/readiness so pre-secret blockers are not assumed
  to be descriptor-only.
- Applied nonzero connect/statement timeout settings through the diagnostic
  executor connection helper.
- Updated Phase 11.4 / Stage C task text with what is now closed and what
  remains open: global cross-query limits, per-remote concurrency caps,
  cancellation, and true async/pipeline execution.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_libpq_executor_budget_limits`
- `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- `git diff --check`

## Review Focus

- Confirm row-granular budget blocking is the right Stage C behavior instead
  of partial PID truncation.
- Confirm `remote_executor_overload` precedence through dispatch, secret, and
  readiness is appropriate for strict-mode fail-closed behavior.
- Confirm the timeout settings are correctly scoped as first diagnostics and
  connection-helper enforcement, while global concurrency/cancellation remain
  open Phase 11 work.
