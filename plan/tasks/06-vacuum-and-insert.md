# Task 06: Vacuum and Online Insert

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
