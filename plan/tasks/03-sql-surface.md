# Task 03: SQL Surface

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

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
