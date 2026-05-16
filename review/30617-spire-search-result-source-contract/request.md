# SPIRE Search Result Source Contract

## Scope

This packet publishes the result-source contract for the final remote-search
coordinator result summary.

Code checkpoint: `7a652318` (`Publish SPIRE search result source contract`)

## Changes

- Adds `ec_spire_remote_search_coordinator_result_contract()`.
- Documents the final search result sources:
  - `local_heap_candidates`
  - `blocked`
  - `none`
- Extends the Phase 7 policy-contract PG18 test to assert the contract row
  count and blocked-result validator.
- Updates the Phase 7 task note with the search result-source contract surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

## Notes

This is a contract-only surface. It does not change coordinator merge, receive,
or heap-resolution behavior.
