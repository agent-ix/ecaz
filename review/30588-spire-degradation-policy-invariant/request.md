# SPIRE Degradation Policy Invariant

## Summary

This packet addresses the 30576/30584 feedback that the static
`ec_spire_remote_degradation_policy_contract()` rows could drift from the
actual remote fanout skip logic.

Changes:

- Adds `remote_degradation_policy_contract_matches_fanout_skip_decisions`, a
  Rust unit test that checks every contract `(consistency_mode,
  placement_state)` pair against `fanout_should_skip_placement(...)`.
- Centralizes the remaining coordinator result-source/finalization literals:
  - `SPIRE_REMOTE_RESULT_SOURCE_LOCAL_HEAP_CANDIDATES`
  - `SPIRE_REMOTE_RESULT_SOURCE_BLOCKED`
  - `SPIRE_REMOTE_FINAL_STATUS_PLANNED`

SQL-visible string values are unchanged.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/root/tests.rs`

## Validation

Head SHA: `cade462f`

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_degradation_policy_contract_matches_fanout_skip_decisions --no-default-features --features pg18`

Result:

- Focused invariant test passed: 1 passed; 0 failed; 1443 filtered out.
- PG18 lib check passed.

## Notes

This packet does not change skip behavior. It adds a drift guard so future
changes to fanout skip/fail behavior must update the SQL-visible contract in
lockstep.
