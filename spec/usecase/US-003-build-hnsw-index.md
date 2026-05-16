---
id: US-003
title: Build HNSW Index on Existing Data
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-001"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/StR-003"
    type: "derives_from"
    cardinality: "N:1"
---
# US-003: Build HNSW Index on Existing Data

**As** a platform engineer,
**I want** to create an HNSW index on a table that already contains `tqvector` data,
**So that** I can enable ANN search on existing data without re-inserting rows.

## Acceptance Criteria

### US-003-AC-1

`CREATE INDEX USING ec_hnsw (tq_code) WITH (m=8, ef_construction=64)` builds a
valid index.

### US-003-AC-2

The build scans all existing heap rows, encodes their `tqvector` codes, and
constructs the HNSW graph.

### US-003-AC-3

After build, queries immediately use the new index.

### US-003-AC-4

The build is crash-safe: a crash mid-build does not corrupt the table or leave
a partial index.
