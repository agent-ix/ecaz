# Review Request: SPIRE Endpoint Diagnostic Contract

## Scope

This packet covers commit `16f3f781d6d536dc5eab31ab47ea24c6295d93d9`.

It addresses reviewer follow-ups from packets 30704 and 30705:

- The endpoint contract now has an explicit
  `direct_sql_endpoint_status_policy` row: direct
  `ec_spire_remote_search` calls may expose non-ready rows for diagnostics, but
  production libpq receive accepts `endpoint_status = ready` only before merge.
- The remote-node model now documents that the v1 FNV-1a
  `profile_fingerprint` is not cryptographic, and that numeric inputs are
  non-negative canonical decimal ASCII with no leading zeroes except `0`.
- The Phase 11 task file records the direct-call diagnostic posture without
  marking broader Stage B complete.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/lib.rs`
- `plan/design/spire-remote-node-model.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

- `cargo fmt`
- `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`
- `git diff --check`

Raw log: `artifacts/cargo-pgrx-pg18-remote-search-receive-contract.log`.

## Reviewer Focus

- Confirm the direct-call diagnostic policy is stated at the right contract
  surface.
- Confirm the fingerprint text closes the 30704 P3 questions without implying
  a stronger cryptographic identity than v1 provides.
