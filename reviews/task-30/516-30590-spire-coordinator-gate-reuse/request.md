# SPIRE Coordinator Gate Reuse

## Summary

This packet reduces the coordinator-summary pipeline redundancy called out in
30584 for the top local-result path.

Changes:

- `remote_search_local_heap_candidate_summary_row(...)` now computes the
  coordinator gate once and projects local candidate summary fields from that
  gate plus a single local candidate-row pass.
- `remote_search_coordinator_result_summary_row(...)` now reuses its already
  computed coordinator gate when deriving the local candidate summary.
- SQL-visible fields and status values are unchanged.

This does not introduce the full future `SpireCoordinatorPipeline` bundle, but
it removes one repeated gate derivation and one repeated local candidate
execution from the composed result-summary path.

## Files

- `src/am/ec_spire/root/hierarchy_snapshots.rs`

## Validation

Head SHA: `b3de01df`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 local_heap`

Result:

- PG18 `local_heap` filter passed 3 tests:
  - `am::ec_spire::tests::remote_local_heap_locator_decode_error_includes_candidate_context`
  - `pg_test_ec_spire_remote_search_local_heap_degraded_skip_status`
  - `pg_test_ec_spire_remote_search_local_heap_resolution_plan`

## Notes

The broader pipeline redundancy finding remains relevant for future composing
surfaces. This slice narrows the highest-level local result summary without
changing the public SQL contract.
