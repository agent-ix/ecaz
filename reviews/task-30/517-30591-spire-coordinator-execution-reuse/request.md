# SPIRE Coordinator Execution Reuse

## Summary

This packet continues the 30584 pipeline-redundancy cleanup by removing an
extra execution-summary walk inside the coordinator gate.

Changes:

- Splits merge-summary projection into
  `remote_search_merge_input_summary_from_execution(...)`.
- Splits finalization projection into
  `remote_search_finalization_summary_from_merge(...)`.
- `remote_search_coordinator_gate_summary_row(...)` now computes
  `remote_search_execution_summary_row(...)` once, then derives merge and
  finalization summaries from that same execution row.

Public SQL-visible fields and status values are unchanged.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`

## Validation

Head SHA: `b387b00a`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 final_summary`

Result:

- PG18 `final_summary` filter passed:
  - `pg_test_ec_spire_remote_search_final_summary_blocked`

## Notes

This is still an incremental cleanup rather than the full future
`SpireCoordinatorPipeline::execute_once(...)` bundle. It removes the duplicated
execution-summary derivation from the gate while preserving the existing public
summary surfaces.
