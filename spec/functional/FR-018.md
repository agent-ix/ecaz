---
id: FR-018
title: Negative Inner Product Wrapper Functions
type: functional-requirement
status: APPROVED
object_type: api
traces:
  - US-002
  - FR-005
  - FR-017
  - FR-006
---
# FR-018: Negative Inner Product Wrapper Functions

## Requirement

The extension SHALL provide negated SQL-visible wrapper functions for ORDER BY semantics over the `<#>` operator family.

### Functions

#### `tqvector_negative_inner_product(a tqvector, b tqvector) -> float4`

- SHALL return `-1 * tqvector_inner_product(a, b)`
- Exists for the commutative `(tqvector, tqvector)` `<#>` operator where ORDER BY ASC means highest similarity first

#### `tqvector_negative_query_inner_product(candidate tqvector, query float4[]) -> float4`

- SHALL return `-1 * tqvector_query_inner_product(candidate, query)`
- Exists for the `(tqvector, float4[])` `<#>` operator used by HNSW ordering and sequential scan

## Acceptance Criteria

### FR-018-AC-1: Negative code-to-code wrapper correctness
For any valid inputs `a` and `b`, `tqvector_negative_inner_product(a, b)` SHALL equal `-1 * tqvector_inner_product(a, b)`.

### FR-018-AC-2: Negative query wrapper correctness
For any valid encoded candidate and raw query, `tqvector_negative_query_inner_product(candidate, query)` SHALL equal `-1 * tqvector_query_inner_product(candidate, query)`.
