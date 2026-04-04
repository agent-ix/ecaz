---
id: FR-005
title: Code-to-Code Inner Product Function
type: functional-requirement
status: APPROVED
object_type: api
traces:
  - US-002
  - FR-013
  - FR-014
  - FR-015
---
# FR-005: Code-to-Code Inner Product Function

## Requirement

The extension SHALL provide a function for computing the symmetric approximate inner product between two stored `tqvector` values using the lower-fidelity code-to-code estimator.

### Function

#### `tqvector_inner_product(a tqvector, b tqvector) -> float4`

- SHALL return the symmetric code-to-code approximate inner product estimate between `a` and `b`
- SHALL reject mismatched `dim` or `bits` with ERROR
- SHALL be `IMMUTABLE`, `STRICT`, `PARALLEL SAFE`
- SHALL use the code-to-code scoring path defined below

### Code-to-Code Scoring Path

Used by the public SQL API `tqvector_inner_product(a, b)` and during HNSW runtime insert/maintenance where both sides are stored compressed codes.

#### Algorithm

`score_ip_encoded_lite` operates directly on packed bytes:
1. Unpack MSE indices from both codes
2. For each dimension, look up `centroid[idx_a] * centroid[idx_b]` from a pre-computed centroid product table
3. Sum -> MSE inner product estimate
4. SHALL NOT apply a QJL correction term in v0.1
5. Return `mse_ip`

Lower fidelity than the prepared-query path because the query is compressed and the residual correction term is omitted, but it avoids query preparation and works symmetrically.

## Acceptance Criteria

### FR-005-AC-1: Known-vector code-to-code accuracy
Given two known 1536-dim vectors encoded at b=4, the code-to-code estimate SHALL be benchmarked against true fp32 inner product using the formulas defined in FR-013 and FR-015.

### FR-005-AC-2: Dimension mismatch error
`tqvector_inner_product(v1536, v768)` SHALL raise ERROR containing "mismatch".

### FR-005-AC-3: Symmetry
`tqvector_inner_product(a, b)` SHALL equal `tqvector_inner_product(b, a)` for all valid inputs.
