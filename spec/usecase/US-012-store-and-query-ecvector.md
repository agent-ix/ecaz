---
id: US-012
title: Store and Query Canonical ecvector Columns
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-028"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-029"
    type: "derives_into"
    cardinality: "1:N"
---
# US-012: Store and Query Canonical ecvector Columns

**As** an application developer,
**I want** to store embeddings in `ecvector(dim)` columns,
**So that** I can use one canonical row type with HNSW, IVF, and DiskANN indexes.

## Acceptance Criteria

### US-012-AC-1

`CREATE TABLE items (embedding ecvector(1536))` accepts finite 1536-dimensional vectors and rejects dimension mismatches.

### US-012-AC-2

`encode_to_ecvector(input, 4, 42)` returns an `ecvector` value usable by all implemented access methods.

### US-012-AC-3

`ORDER BY embedding <#> $query LIMIT k` works through the appropriate opclass for each implemented AM.
