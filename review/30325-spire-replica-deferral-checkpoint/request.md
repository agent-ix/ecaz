# SPIRE Replica Deferral Checkpoint

## Checkpoint

- Code commit: `49c9b9b8`
  (`Record SPIRE replica deferral checkpoint`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Phase 0 storage-design consistency for replica deferral

## Summary

This checkpoint makes the already-documented replica deferral explicit in the
Phase 0 decision list and marks the corresponding Task 30 plan item complete.

- The Phase 0 storage note now lists replicated partition objects as deferred
  to a future boundary-replica/remote availability phase.
- The plan now records that Phase 1 publishes one primary placement per PID.
- No code, storage format, scan behavior, placement semantics, or remote
  protocol behavior changed.

## Changed Files

- `plan/design/spire-phase0-partition-object-storage.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`
- `git diff --cached --check` before commit

Tests were not run because this is a documentation-only checkpoint.

## Notes

- Boundary-replica assignment fanout, replica dedupe, remote placement, and
  read-throughput/availability replicas remain future work.
- This checkpoint only closes the Phase 0 requirement to record the deferral
  before continuing persistence and scan/vacuum implementation slices.
