---
id: US-014
title: Build and Tune DiskANN Indexes
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-034"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-035"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-036"
    type: "derives_into"
    cardinality: "1:N"
---
# US-014: Build and Tune DiskANN Indexes

**As** a platform engineer,
**I want** to create and tune `ec_diskann` indexes,
**So that** I can compare Vamana/DiskANN behavior against HNSW and external DiskANN references.

## Acceptance Criteria

### US-014-AC-1

`CREATE INDEX ... USING ec_diskann` builds a persisted Vamana graph over finite unit-normalized `ecvector` rows.

### US-014-AC-2

The user can tune graph degree, build list size, scan list size, rerank budget, top-k, alpha, and traversal prefilter.

### US-014-AC-3

DiskANN build, scan, insert, vacuum, diagnostics, and benchmark surfaces expose enough state to reproduce landed local measurements.
