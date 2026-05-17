---
id: US-007
title: Planner-Visible Cost Model
type: user-story
artifact_type: US
status: DRAFT
priority: P1-critical
relationships:
  - target: "ix://agent-ix/ecaz/StR-004"
    type: "derives_from"
    cardinality: "N:1"
---
# US-007: Planner-Visible Cost Model

**As** an application developer,
**I want** the query planner to automatically choose the HNSW index for `ORDER BY <#> LIMIT k` queries without manual hints,
**So that** ANN search works through the standard Postgres query planner like any other index.

## Acceptance Criteria

### US-007-AC-1

`EXPLAIN SELECT ... ORDER BY col <#> $q LIMIT 10` shows "Index Scan using
ec_hnsw" on a table with an HNSW index.

### US-007-AC-2

The planner cost model accounts for graph traversal pages, linear scan pages,
and CPU scoring cost.

### US-007-AC-3

On PG18, `amgettreeheight` reports the HNSW graph's `max_level` to the planner
for cost refinement.

### US-007-AC-4

The planner correctly prefers sequential scan over index scan for very small
tables with fewer than 100 rows.
