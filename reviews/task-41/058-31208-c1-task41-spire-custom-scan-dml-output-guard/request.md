# Review Request: Task 41 SPIRE CustomScan DML Output Guard

## Summary

This slice migrates SPIRE CustomScan DML output loading from a manual
`index_open` / `index_close` pair to `IndexRelationGuard`, and narrows the
executor-state unsafe access around owned field copies and final assignment.

Code commit: `f5bcf0fd3dce31e55281829d43a97670d9a28696`

## Changes

Updated `src/am/ec_spire/custom_scan/dml.rs`:

- `custom_scan_ensure_outputs` now opens the index relation through
  `IndexRelationGuard::access_share`.
- The state fields needed for remote output loading are copied under a
  documented unsafe read before the remote stream call.
- The final state mutation is kept as a narrow documented unsafe write.
- Removed the direct `index_open` / `index_close` pair from the helper.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: `4141`
- After: `4140`

## Review Focus

- Confirm the guard lifetime covers
  `remote_search_production_scan_tuple_payload_result_stream`.
- Confirm cloning `query` and `tuple_payload_columns` preserves behavior while
  avoiding borrowed executor-state references across possible PostgreSQL error
  paths.
- Confirm the final state write is equivalent to the previous loaded-output
  transition.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
