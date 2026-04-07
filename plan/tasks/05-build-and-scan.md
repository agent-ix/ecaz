# Task 05: Build and Scan

Status: in progress

Progress notes:
- Build path is implemented and validated.
- Scan lifecycle, query payload ownership, metadata caching, prepared-query caching, and bootstrap
  linear tuple production are implemented.
- Graph traversal, distance ordering, and planner enablement remain pending.
- `ef_search` now has a split control surface direction: relation default plus session override,
  while planner-visible scans remain disabled per ADR-011.
- Planner/admin scaffolding may now expose read-only effective tuning and current live-node counts
  without enabling planner-visible scans.

## Scope

Implement bulk build and indexed query execution for `tqhnsw`.

## Owns

- `FR-008`
- `FR-009`

## Dependencies

- Task 01
- Task 03
- Task 04

## Unblocks

- end-to-end indexed ANN search
- recall benchmarking
- vacuum and insert validation on realistic indexes

## Deliverables

- `ambuild`
- `ambuildempty`
- `amoptions`
- `ambeginscan`
- `amrescan`
- `amgettuple`
- `amendscan`
- `ef_search` behavior

## Primary Tests

- `TC-112` to `TC-125`
- `TC-131`

## Notes

- Split implementation internally if needed:
  - build serialization
  - scan traversal
- Keep build and scan in separate modules even if the same worker owns both.
