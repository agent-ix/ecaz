# Review Request: SPIRE Libpq Receive Attempt Diagnostics

## Scope

This packet covers commit `29b130f3f751e567dd9156d45fca76ba0b685fe0`.

The slice adds `ec_spire_remote_search_libpq_executor_receive_attempts(...)`,
an operator-facing per-node diagnostic surface for remote receive failures. It
does not relax the production executor: `ec_spire_remote_search_libpq_executor_candidates`
still fails closed before merge on a non-ready endpoint. The new surface reports:

- `status`: named mismatch, including `requires_rabitq_storage_format`.
- `next_blocker`: the failing contract area, such as `remote_endpoint_identity`.
- `failure_action`: `fail_closed` in strict mode or `skip_node` in degraded mode.
- `failure_reason`: exact receive/decode error text.

This is the first concrete Stage B/E bridge for degraded skip reporting: strict
behavior remains paired with the exact degraded skip reason.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `cargo fmt`
- `cargo pgrx test pg18 test_ec_spire_libpq`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

Raw logs are in `artifacts/`; see `artifacts/manifest.md`.

## Reviewer Focus

- Confirm the new surface is diagnostic-only and does not alter the production
  executor fail-closed behavior.
- Confirm `failure_action = skip_node` is the right degraded-mode vocabulary
  for Stage E fault-matrix reporting.
- Confirm the endpoint mismatch status extraction is acceptable as a first
  named reason before broader fingerprint/opclass/version-skew cases land.
