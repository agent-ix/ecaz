# Review Request: SPIRE Selected Execution Input Validators

## Summary

Task 30 SPIRE Phase 2 now has selected-plan validators for relation and local
scheduled replacement execution inputs.

Changes:
- Add `validate_relation_selected_scheduled_replacement_execution_publish_plan`.
- Add `validate_local_selected_scheduled_replacement_execution_publish_plan`.
- Keep decision, PID plan, and publish plan bundled during final execution-input
  drift checks.
- Cover successful validation and `next_local_vec_seq` drift for both relation
  and local inputs.
- Update the Phase 2 checklist.

## Validation

- `cargo test selected_scheduled_execution_publish_plan_validators --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure selected-plan
validation slice.
