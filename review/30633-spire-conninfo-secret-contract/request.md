# SPIRE Conninfo Secret Contract

## Scope

Task 30 SPIRE Phase 7 now has an explicit SQL-visible authentication/secret
resolution decision for libpq executor work.

Code checkpoint: `8242594d` (`Add SPIRE conninfo secret contract`)

## Changes

- Added `ec_spire_remote_conninfo_secret_resolution_contract()`.
- Selected `external_executor_secret_provider` for v1.
- Documented that SQL/catalog surfaces store only `conninfo_secret_name`, never
  raw conninfo.
- Marked `postgres_fdw_user_mapping` as not selected for v1 and
  `in_extension_conninfo_table` as rejected for v1.
- Extended the Phase 7 policy contract PG test to assert the selected provider,
  raw-conninfo rejection, and extension-table rejection.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

## Review Focus

- Whether `external_executor_secret_provider` is the right v1 decision for
  unblocking the libpq executor.
- Whether the contract is explicit enough that future executor work cannot
  accidentally introduce raw conninfo storage or SQL exposure.
