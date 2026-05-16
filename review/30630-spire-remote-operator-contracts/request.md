# SPIRE Remote Operator Contracts

## Scope

Task 30 SPIRE Phase 7 now exposes two additional SQL-visible contract surfaces
for the remote coordinator handoff:

- `ec_spire_remote_operator_entrypoint_contract()` names the compact
  operator-facing subset of the larger remote diagnostic surface.
- `ec_spire_remote_libpq_connection_lifecycle_contract()` names the current
  libpq executor lifecycle policy for remote search and remote manifest
  publication.

Code checkpoint: `1e17c28d` (`Add SPIRE remote operator contracts`)

## Changes

- Added static contract row types and helpers for remote operator entrypoints
  and libpq connection lifecycle policy.
- Added SQL wrappers for both contracts.
- Extended the Phase 7 policy contract PG test to assert the entrypoint set,
  the search/manifest lifecycle rows, per-query/no-pooling policy, executor
  secret resolution, and no raw conninfo SQL exposure.
- Updated `plan/tasks/30-spire-ivf-foundation.md` with the new operator and
  lifecycle contract status.

## Validation

- `cargo fmt`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

## Review Focus

- Whether the eight named operator entrypoints are the right compact subset for
  Phase 7 operations.
- Whether the lifecycle contract is explicit enough for the future libpq
  executor: per-query connections, no pooling in v1, executor-owned secret
  resolution, no raw conninfo exposure through SQL, and fail-closed/no implicit
  retry.
