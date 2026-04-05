# Task 04: Page Layout and WAL

Status: complete

## Scope

Implement page tuple layouts, fit invariants, and GenericXLog-backed write helpers for the custom access method.

## Owns

- `FR-007`
- `FR-011`

## Dependencies

- Task 02 for persisted payload conventions

## Unblocks

- bulk build
- scan
- vacuum
- online insert

## Deliverables

- Metadata page format
- `TqElementTuple`
- `TqNeighborTuple`
- level-cap calculation
- page-fit checks
- WAL helper abstractions

## Primary Tests

- `TC-034`
- `TC-117`
- `TC-119`
- `TC-126`
- `TC-127`

## Notes

- This is one of the highest-risk shared interfaces. Stabilize it before parallel AM work fans out.
