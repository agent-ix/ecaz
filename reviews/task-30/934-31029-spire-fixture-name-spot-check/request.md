# Review Request: SPIRE fixture name spot-check evidence

## Summary

This checkpoint closes the Phase 12b fixture-name spot-check row.

The packet selected ten fixture strings from the Phase 11/12/12a/12b trackers
and checked each against `src/tests/` with fixed-string `rg`. All ten selected
strings still resolve after the fixture module split.

Code checkpoint: `5aa4dbdf1d337e5b6621d7e659edaaa1aaaadf15`

## Selected Strings

- `test_ec_spire_schema_drift_fails_before_dispatch_sql`
- `test_ec_spire_placement_index_oid_lookup_uses_index_sql`
- `test_ec_spire_customscan_does_not_replace_local_only_index_plan`
- `test_ec_spire_enable_coordinator_insert_trigger_sql`
- `test_ec_spire_prod_transport_local_cancel_remote_cancel`
- `test_ec_spire_insert_prepare_local_cancel_rolls_back`
- `test_ec_spire_prod_receive_local_cancel_remote_cancel`
- `test_ec_spire_dml_frontdoor_primitive_plan_from_decision`
- `test_ec_spire_srcid`
- `test_ec_spire_update_delete_schema_drift_guard_sql`

`test_ec_spire_srcid` is a tracker filter string rather than one exact fixture
function; it resolves to the source-identity fixture family in
`src/tests/insert.rs`.

## Validation

- `selected-fixture-names.log`: selected strings
- `fixture-location-check.log`: all ten strings resolved under `src/tests/`
- `git diff --check` for the tracker update

Raw logs and command metadata are in `artifacts/manifest.md`.

## Reviewer Focus

- Confirm the selected strings are representative enough for the requested
  random spot-check.
- Confirm the `test_ec_spire_srcid` filter-string handling is acceptable.
