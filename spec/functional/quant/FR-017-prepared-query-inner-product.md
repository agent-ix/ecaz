---
id: FR-017
title: Prepared-Query Inner Product Function
type: FR
status: APPROVED
object: api_endpoint
traces:
  - US-002
  - FR-013
  - FR-014
  - FR-015
---
# FR-017: Prepared-Query Inner Product Function

## Description

The extension SHALL provide a function for computing the approximate inner product between a stored `tqvector` candidate and a raw fp32 query using the prepared-query LUT estimator.

### Function

#### `tqvector_query_inner_product(candidate tqvector, query float4[]) -> float4`

- SHALL return the prepared-query approximate inner product estimate between a stored `tqvector` candidate and a raw fp32 query
- SHALL reject dimension mismatch with ERROR
- SHALL be `IMMUTABLE`, `STRICT`, `PARALLEL SAFE`
- SHALL use the prepared-query scoring path defined below

### Prepared-Query LUT Scoring Path

Used by the HNSW scan implementation (FR-009) and by `tqvector_query_inner_product(candidate, query)`.

#### LUT Structure

The lookup table has shape `[dim] x [num_centroids]` where `num_centroids = 2^(bits-1)`:

Let:
- `d` = original persisted dimension
- `n` = internal transform dimension `next_power_of_two(d)`
- `K = 2^(bits-1)` = number of centroids
- `y_q in R^n` = SRHT-rotated query
- `s_q in R^n` = QJL projection of the raw query under the QJL seed

The prepared-query state SHALL retain the first `d` coordinates of both transforms:

```
lut[i][c] = centroid[c] * y_q[i],   for i in [0, d), c in [0, K)
sq[i]     = s_q[i],                 for i in [0, d)
qjl_scale = sqrt(pi / 2) / d
```

- `dim` = original persisted dimension
- `num_centroids` = number of MSE codebook centroids (e.g., 8 for 4-bit)
- Each entry is f32
- Memory footprint: `dim x num_centroids x 4` bytes (e.g., 1536 x 8 x 4 = 48 KB for 1536-dim 4-bit)

The prepared query state SHALL also retain:
- the `sq` vector of projected query coordinates for the first `d` positions
- the scalar `qjl_scale = sqrt(pi / 2) / d`

#### LUT Scoring Algorithm

This section is the normative behavioral contract for the raw-query scoring path. Implementation-facing APIs in FR-015 SHALL implement this algorithm and SHALL NOT redefine conflicting math.

For each candidate code:
1. Unpack the candidate's MSE indices (bit-packed -> array of CodeIndex)
2. For each dimension `i`, look up `lut[i][mse_index[i]]` - one table lookup per dimension
3. Sum all lookups -> MSE inner product estimate
4. Unpack the candidate's persisted `gamma`
5. Compute `qjl_sum = sum_{i=0}^{d-1} sq[i] * sign(candidate_qjl[i])`
6. Compute `qjl_correction = gamma * qjl_scale * qjl_sum`
7. Return `mse_ip + qjl_correction`

**O(dim) per candidate, zero heap allocation per scoring call.** The LUT is constructed once per query and reused for all candidates.

### Sequential Scan Usage

When PostgreSQL executes `ORDER BY tqvector_query_inner_product(candidate, query)` or `candidate <#> query` without an index, evaluation is row-by-row through the SQL function contract above. This path is approximate and throughput-bound. The extension SHALL NOT rely on hidden scan-local state inside the immutable SQL function. Any future prepared-query reuse for sequential scan SHALL be introduced through an explicit executor integration or separate API.
The specification treats this row-by-row query preparation cost as an accepted v0.1 performance tradeoff. Sequential scan throughput SHALL therefore be characterized explicitly by benchmarks rather than assumed to match index-scan prepared-query reuse.

## Endpoint

SQL-callable C function declared in `sql/bootstrap.sql` (backed by the
`tqvector_query_inner_product_wrapper` symbol):

```sql
tqvector_query_inner_product(tqvector, real[]) RETURNS float4
  IMMUTABLE STRICT PARALLEL SAFE LANGUAGE c
```

- **Arguments**: a stored `tqvector` candidate and a raw fp32 `real[]`
  (`float4[]`) query.
- **Returns**: `float4`, the prepared-query (query-to-code) inner product estimate.
- **Behavior**: builds prepared-query state via
  `ProdQuantizer::prepare_ip_query` (SRHT-rotated query LUT
  `lut[i][c] = centroid[c] * y_q[i]`, the QJL projection `sq`, and
  `qjl_scale = sqrt(pi/2) / d`), then scores the candidate with
  `score_ip_encoded`. The result is the MSE LUT sum plus the gamma-weighted QJL
  term `gamma * qjl_scale * sum(sq[i] * sign(qjl[i]))`, dispatched through the
  SIMD path (FR-014) with a scalar fallback. Dimension mismatch raises ERROR.
  Marked `IMMUTABLE`, `STRICT`, `PARALLEL SAFE`; the immutable contract carries
  no scan-local state, so each row re-prepares the query under sequential scan.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-017-AC-1 | Query-to-code estimates for a known 1536-dim query and b=4 candidate are benchmarked against true fp32 inner product per FR-013/FR-015 formulas | Analysis |
| FR-017-AC-2 | `tqvector_query_inner_product(v1536, q768)` raises ERROR containing "mismatch" | Test |
| FR-017-AC-3 | After prepared-query setup, `score_ip_encoded` does not allocate heap memory per call | Analysis |
| FR-017-AC-4 | The LUT for 1536-dim 4-bit occupies <= 48 KB | Analysis |

### FR-017-AC-1: Known-vector query-to-code accuracy
Given a known 1536-dim raw query vector and a known encoded candidate at b=4, the query-to-code estimate SHALL be benchmarked against true fp32 inner product using the formulas defined in FR-013 and FR-015.

### FR-017-AC-2: Dimension mismatch error
`tqvector_query_inner_product(v1536, q768)` SHALL raise ERROR containing "mismatch".

### FR-017-AC-3: Zero allocation in prepared-query hot path
After prepared-query setup, `score_ip_encoded` SHALL not allocate heap memory (verified by benchmarking, no per-call Vec or Box).

### FR-017-AC-4: LUT memory footprint
The LUT for 1536-dim 4-bit SHALL occupy <= 48 KB.

## Dependencies

- **Upstream**: US-002 (traces), FR-013 (estimator contract), FR-014 (SIMD acceleration), FR-015 (`score_ip_encoded` implementation)
- **Downstream**: FR-018 (negated wrapper over this function)
