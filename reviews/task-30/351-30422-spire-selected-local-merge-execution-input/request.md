# Review Request: SPIRE Selected Local Merge Execution Input

## Summary

Task 30 SPIRE Phase 2 now has a local dry-run merge execution-input helper that
consumes the selected publish-lock plan directly.

Changes:
- Add `build_local_selected_scheduled_merge_replacement_execution_input`.
- Keep the chosen merge decision, PID plan, and publish plan bundled until
  local execution-input construction.
- Preserve caller-provided placement-write evidence.
- Cover successful selected merge planning and split-plan rejection.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_selected_scheduled_merge_replacement_execution_input --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This keeps local dry-run merge
composition aligned with the relation selected-plan helper.
