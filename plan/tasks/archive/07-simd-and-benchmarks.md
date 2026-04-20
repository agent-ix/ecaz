# Legacy Task 07: SIMD and Benchmarks

Status: archived legacy snapshot

Superseded by:
- `plan/tasks/08-simd.md`
- `plan/tasks/10-benchmarks.md`

## Scope

Optimize the stable scalar implementation and run the required quality/performance benchmarks.

## Owns

- `FR-014`
- `NFR-001`
- `NFR-002`
- `NFR-003`

## Dependencies

- Task 01
- Task 05
- Task 06

## Unblocks

- performance sign-off
- quality sign-off
- storage and recall claims

## Deliverables

- AVX2+FMA implementations
- NEON implementations
- scalar fallback validation
- benchmark harnesses and reproducible scripts

## Primary Tests

- `TC-016`
- `TC-017`
- `TC-030`
- `BC-001` to `BC-016` as applicable

## Notes

- Do not start aggressive optimization before scalar APIs and outputs are stable.
