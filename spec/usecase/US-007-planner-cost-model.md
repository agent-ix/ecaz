---
id: US-007
title: Planner-Visible Cost Model
type: user-story
status: DRAFT
priority: P1-critical
traces:
  - StR-004
---
# US-007: Planner-Visible Cost Model

**As** an application developer,
**I want** the query planner to automatically choose the HNSW index for `ORDER BY <#> LIMIT k` queries without manual hints,
**So that** ANN search works through the standard Postgres query planner like any other index.

## Acceptance Criteria

1. `EXPLAIN SELECT ... ORDER BY col <#> $q LIMIT 10` shows "Index Scan using ec_hnsw" on a table with an HNSW index
2. The planner cost model accounts for graph traversal pages, linear scan pages, and CPU scoring cost
3. On PG18, `amgettreeheight` reports the HNSW graph's `max_level` to the planner for cost refinement
4. The planner correctly prefers sequential scan over index scan for very small tables (< 100 rows)
