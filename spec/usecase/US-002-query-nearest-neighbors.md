---
id: US-002
title: Query Nearest Neighbors via SQL
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-001"
    type: "derives_from"
    cardinality: "N:1"
---
# US-002: Query Nearest Neighbors via SQL

**As** an application developer querying agent memories,
**I want** to find the top-k nearest vectors using standard SQL ORDER BY with the `<#>` operator and a raw query embedding,
**So that** ANN search works through the standard Postgres query planner with no application-side logic.

## Acceptance Criteria

### US-002-AC-1

`SELECT * FROM memories ORDER BY tq_code <#> $query_embedding LIMIT 10`
returns the approximate 10 nearest neighbors, where `$query_embedding` is
`float4[]`.

### US-002-AC-2

The query uses the HNSW index, confirmed via EXPLAIN.

### US-002-AC-3

Results are ordered by ascending negative inner product, with highest
similarity first.

### US-002-AC-4

The query completes within the latency bounds defined in `NFR-001`.
