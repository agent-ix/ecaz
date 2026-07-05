# Review Request: SPIRE Local Scheduled Split Execution Input

## Summary

Task 30 SPIRE Phase 2 now has local dry-run helpers that compose scheduled
split routing/leaf preparation into local scheduled replacement execution
parts and final execution input.

Changes:
- Add `build_local_scheduled_split_replacement_execution_parts`.
- Add `build_local_scheduled_split_replacement_execution_input`.
- Reuse relation split composition so local and relation split validation stay
  in lockstep, while preserving caller-provided placement-write evidence.
- Cover write-evidence preservation, publish-plan binding, leaf-input ordering,
  and next-PID drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_scheduled_split_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. Split centroid training and
routed leaf-input production remain live scheduler responsibilities.
