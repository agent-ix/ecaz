# SPIRE Search Empty Result Source Coverage

## Scope

This packet covers the `none` result-source state for
`ec_spire_remote_search_coordinator_result_summary(...)`.

Code checkpoint: `efb5c154` (`Cover SPIRE search empty result source`)

## Changes

- Adds PG18 coverage for a local coordinator result with `top_k = 0`.
- Asserts that the final result summary reports:
  - `result_source = 'none'`
  - `returned_candidate_count = 0`
  - `next_blocker = 'none'`
  - `status = 'empty_top_k'`
- Corrects the search result-source contract wording for the `none` state from
  ready-empty to the observed `empty_top_k` status family.
- Keeps the contract validator tied to zero returned candidates plus no
  blocker.

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
- `cargo pgrx test pg18 test_ec_spire_remote_search_coordinator_result_ready_empty`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

## Notes

The first validation attempt showed the actual final status is `empty_top_k`;
the contract and assertion now reflect that observed behavior.
