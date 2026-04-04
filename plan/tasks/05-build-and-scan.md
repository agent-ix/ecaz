# Task 05: Build and Scan

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
