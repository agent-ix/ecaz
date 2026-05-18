# Review Request: SPIRE Cost/Planner Fixture Split

## Summary

This cleanup slice creates `src/tests/cost_and_planner.rs` and moves the
remaining SPIRE planner-facing registration fixtures out of
`src/tests/mod.rs`:

- `test_ec_spire_access_method_is_registered`
- `test_ec_spire_operator_classes_are_registered`
- `test_ec_spire_custom_scan_status_registered_fail_closed`

The Phase 12b tracker now marks `tests/cost_and_planner.rs` closed.

Code commit: `4eba0c3b1ba2c819cca5264f007d19035fe0db6a`

## Validation

Packet-local logs are in `artifacts/`.

Passing checks:

- `cargo fmt --check`
- `git diff --check`
- location check confirms the moved registration/status fixtures now live in
  `src/tests/cost_and_planner.rs`
- PG18 focused tests:
  - `test_ec_spire_access_method_is_registered`
  - `test_ec_spire_operator_classes_are_registered`
  - `test_ec_spire_custom_scan_status_registered_fail_closed`

## Review Focus

Please check that:

- the moved fixture bodies are unchanged;
- `cost_and_planner.rs` is an appropriate concern file for AM/opclass and
  CustomScan hook/status registration checks;
- closing `tests/cost_and_planner.rs` is appropriate.
