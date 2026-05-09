# Review Request: SPIRE Pipeline Steps Consolidation

## Summary

Task 30 Phase 8 preflight now has a consolidated remote pipeline diagnostic:
`ec_spire_remote_pipeline_steps(...)`.

Code checkpoint: `544d2512` (`Add SPIRE remote pipeline steps`)

## Scope

- Adds `ec_spire_remote_pipeline_steps(index_oid, requested_epoch, query,
  selected_pids, top_k, consistency_mode)`.
- Returns one row per high-level remote pipeline step:
  `dispatch_plan`, `connection_check`, `candidates`, `heap_candidates`,
  `manifest_apply`, and `coordinator_result`.
- Keeps the surface diagnostic-only. It reuses existing dispatch, connection,
  executor candidate, heap-candidate, manifest-publication, and coordinator
  result status/count contracts without changing remote execution semantics.
- Adds the new surface to `ec_spire_remote_operator_entrypoint_contract()`.
- Extends PG18 coverage so consolidated rows are compared against the existing
  per-surface status/count contracts in the loopback executor fixture.

## Validation

- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
  - `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
  - `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`
- `git diff --check`

## Notes

The reviewed shorthand asked for `ec_spire_remote_pipeline_steps(index_oid)`.
The implemented SQL surface takes the same request-shaped arguments as the
existing remote-search diagnostics because dispatch, candidate, heap, and final
result statuses are query/selected-PID/top-k dependent.
