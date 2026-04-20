---
id: US-003
title: Build HNSW Index on Existing Data
type: user-story
status: APPROVED
traces:
  - StR-001
  - StR-003
---
# US-003: Build HNSW Index on Existing Data

**As** a platform engineer,
**I want** to create an HNSW index on a table that already contains `tqvector` data,
**So that** I can enable ANN search on existing data without re-inserting rows.

## Acceptance Criteria

1. `CREATE INDEX USING ec_hnsw (tq_code) WITH (m=8, ef_construction=64)` builds a valid index
2. The build scans all existing heap rows, encodes their tqvector codes, and constructs the HNSW graph
3. After build, queries immediately use the new index
4. The build is crash-safe — a crash mid-build does not corrupt the table or leave a partial index
