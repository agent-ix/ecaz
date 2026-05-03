# SPIRE Scan Path Checklist Scope

## Checkpoint

- Code commit: `ffcf5402`
  (`Clarify SPIRE scan path checklist scope`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Round review follow-up for packet `30330`

## Summary

This checkpoint aligns the Task 30 checklist with the scan-path caveats already
recorded in prose.

The checked scan-path item now explicitly applies to populated TurboQuant and
RaBitQ SPIRE indexes. Populated PQ-FastScan scan support is split into a
separate unchecked item that depends on persisted grouped-PQ model metadata and
scan-time scorer binding.

This keeps the checklist truth aligned with the implementation state: empty
`pq_fastscan` SPIRE indexes already scan safely because there are no
assignments to score, but populated `pq_fastscan` SPIRE indexes remain
build-blocked.

## Changed Files

- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`

Tests were not run because this is a documentation-only review follow-up.

## Notes

- This responds to the round-review recommendation to avoid marking the full
  scan path complete while populated PQ-FastScan remains explicitly deferred.
