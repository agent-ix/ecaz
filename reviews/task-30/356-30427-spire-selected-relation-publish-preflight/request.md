# Review Request: SPIRE Selected Relation Publish Preflight

## Summary

Task 30 SPIRE Phase 2 now has selected-plan preflight validation for relation
scheduled replacement publish plus a selected-plan relation publish wrapper.

Changes:
- Add `validate_relation_selected_scheduled_replacement_publish_inputs`.
- Add `publish_relation_selected_scheduled_replacement_epoch`.
- Keep selected decision, PID plan, and publish plan bundled through relation
  publish validation before delegating to the existing relation publish path.
- Cover successful preflight plus previous-manifest and execution-input drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test relation_selected_scheduled_replacement_publish_inputs --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. The relation publish wrapper is
an unsafe callback-facing shim over the existing relation write path; the new
preflight is pure and unit-covered.
