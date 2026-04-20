---
id: FR-006
title: SQL Operators and Operator Class
type: functional-requirement
status: APPROVED
object_type: api
traces:
  - US-002
  - FR-017
  - FR-018
---
# FR-006: SQL Operators and Operator Class

## Requirement

The extension SHALL register SQL operators and an operator class for HNSW index integration.

### Code-to-Code Operator: `<#>`

```sql
CREATE OPERATOR <#> (
    leftarg   = tqvector,
    rightarg  = tqvector,
    procedure = tqvector_negative_inner_product,
    commutator = <#>
);
```

- Returns negative inner product (for ORDER BY ASC = highest similarity first)
- Commutative: `a <#> b = b <#> a`

### Query Operator: `<#>`

```sql
CREATE OPERATOR <#> (
    leftarg   = tqvector,
    rightarg  = float4[],
    procedure = tqvector_negative_query_inner_product
);
```

- Returns negative query-to-code inner product
- Used by the HNSW operator class and by sequential scan over raw query vectors

### Operator Class: `tqvector_ip_ops`

```sql
CREATE OPERATOR CLASS tqvector_ip_ops DEFAULT FOR TYPE tqvector
    USING ec_hnsw AS
    OPERATOR 1 <#>(tqvector, float4[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, float4[]);
```

- Default operator class for `tqvector` under the `ec_hnsw` access method
- OPERATOR 1 is the ordering operator for raw query vectors
- FUNCTION 1 is the prepared-query distance function used by the index AM

## Acceptance Criteria

### FR-006-AC-1: Operator usable in ORDER BY
`SELECT * FROM t ORDER BY col <#> $query LIMIT 10` SHALL parse and execute when `$query` is `float4[]`.

### FR-006-AC-2: Index scan chosen
EXPLAIN of the above query on an indexed table SHALL show an Index Scan using `ec_hnsw`.

Current staged behavior:
- Until ADR-011 is retired, planner/explain snapshot helpers MAY report why `ec_hnsw` is still
  gated off, but EXPLAIN itself is not yet expected to show a `ec_hnsw` index scan.
- Explain-facing snapshot helpers MAY also report the intended `<#>` ordering semantics
  (`strategy 1` / `COMPARE_LT`) and that PG18 strategy-translation callbacks are still unavailable.
- Explain-facing snapshot helpers MAY also report the intended custom EXPLAIN option name
  (`tqvector`) and that PG18 EXPLAIN option / hook wiring is still unavailable.

### FR-006-AC-3: Operator commutativity
`a <#> b` SHALL equal `b <#> a` for the `(tqvector, tqvector)` overload.
