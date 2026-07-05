# Review Request: SPIRE Scheduled Replacement Object Writer

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: decision-bound wrappers for writing scheduled replacement objects.

## Summary

- Added `write_local_scheduled_replacement_objects`.
- Added `write_relation_scheduled_replacement_objects`.
- The wrappers validate scheduler decision shape, parent PID consistency, and
  replacement-child count before delegating to the existing replacement object
  writer validation and storage paths.
- Added focused local-store tests for successful scheduled object writes,
  wrong-parent rejection, and child-count mismatch rejection.
- Updated the Task 30 Phase 2 checklist to record this scheduler execution
  preparation slice.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test local_scheduled_replacement_object_writer --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

- The relation wrapper is intentionally not PG-tested in this slice; it reuses
  the same generic writer path covered through the local-store wrapper.
- This still does not schedule work or recompute centroids. It provides the
  decision-bound object-write step that live scheduler execution can call after
  routing rewrite and leaf-input assembly.
- No measurement claims are made in this packet.
