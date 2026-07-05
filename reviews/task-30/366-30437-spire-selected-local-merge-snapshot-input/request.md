# Review Request: SPIRE Selected Local Merge Snapshot Input

## Summary

Task 30 SPIRE Phase 2 now has a local merge execution-input helper that loads
selected materials from the active snapshot before local input construction.

Changes:
- Add `build_local_selected_scheduled_merge_replacement_execution_input_from_snapshot`.
- Load the selected parent routing object through the selected publish-lock
  plan.
- Collect folded affected-leaf rows through the selected publish-lock plan.
- Preserve caller-provided placement-write evidence for local dry-run inputs.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_selected_scheduled_merge_replacement_execution_input_from_snapshot --lib`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure local
material-loading helper before object writes or draft assembly.
