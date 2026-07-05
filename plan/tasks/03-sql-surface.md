# Task 03: SQL Surface

Status: complete

## Scope

Implement the public SQL-callable encode and scoring functions plus operator and packaging support.

## Owns

- `FR-004`
- `FR-005`
- `FR-006`
- `FR-012`
- `FR-017`
- `FR-018`

## Dependencies

- Task 01
- Task 02

## Unblocks

- SQL usability
- HNSW operator-class wiring
- install/uninstall verification

## Deliverables

- `encode_to_tqvector`
- `tqvector_inner_product`
- `tqvector_query_inner_product`
- negative wrapper functions
- `<#>` operators
- operator class
- extension SQL packaging

## Primary Tests

- `TC-105` to `TC-110`
- `TC-114`
- `TC-116`
- `TC-130`
- `TC-134`
- `BC-003`

## Notes

- Keep wrapper semantics trivial and test-locked.
- Packaging can proceed in parallel once symbol names are stable.
