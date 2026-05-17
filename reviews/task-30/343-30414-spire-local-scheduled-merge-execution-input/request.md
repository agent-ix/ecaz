# Review Request: SPIRE Local Scheduled Merge Execution Input

## Summary

Task 30 SPIRE Phase 2 now has local dry-run helpers that compose scheduled
merge routing/leaf preparation into local scheduled replacement execution
parts and final execution input.

Changes:
- Add `build_local_scheduled_merge_replacement_execution_parts`.
- Add `build_local_scheduled_merge_replacement_execution_input`.
- Reuse relation merge composition so local and relation merge validation stay
  in lockstep, while preserving caller-provided placement-write evidence.
- Cover write-evidence preservation, publish-plan binding, and next-PID drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test local_scheduled_merge_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a pure helper slice for
local scheduler dry-run execution once merge replacement rows are available.
