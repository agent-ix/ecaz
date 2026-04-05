# Task 06: Vacuum and Online Insert

Status: in progress

Progress notes:
- Live insert now supports shape validation, first-insert metadata initialization, duplicate
  coalescing, and tail-page append/reuse.
- Vacuum callbacks are currently benign no-ops.
- Graph-aware insertion, drift statistics, and vacuum graph repair remain pending.

## Scope

Implement maintenance paths and insert-drift observability.

## Owns

- `FR-010`
- `FR-016`

## Dependencies

- Task 01
- Task 04
- Task 05

## Unblocks

- maintenance correctness
- drift measurement
- post-build lifecycle completeness

## Deliverables

- `ambulkdelete`
- `amvacuumcleanup`
- `aminsert`
- insert-drift metadata/statistics exposure

## Primary Tests

- `TC-115`
- `TC-118`
- `TC-128`
- `TC-132`
- `TC-133`
- `BC-011`
- `BC-016`

## Notes

- Vacuum and insert should share page-update utilities rather than duplicating mutation logic.
