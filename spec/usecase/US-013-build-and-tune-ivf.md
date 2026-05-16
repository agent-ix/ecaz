---
id: US-013
title: Build and Tune IVF Indexes
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-031"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-032"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-033"
    type: "derives_into"
    cardinality: "1:N"
---
# US-013: Build and Tune IVF Indexes

**As** a platform engineer,
**I want** to create and tune `ec_ivf` indexes on `ecvector` data,
**So that** I can measure posting-list tradeoffs for recall, latency, storage, and ingest.

## Acceptance Criteria

### US-013-AC-1

`CREATE INDEX ... USING ec_ivf` builds an index with deterministic centroid training and persisted posting lists.

### US-013-AC-2

The user can tune `nlists`, `nprobe`, `storage_format`, `pq_group_size`, `rerank`, and `rerank_width`.

### US-013-AC-3

IVF scan, insert, vacuum, admin, and cost surfaces expose enough state to reproduce landed local measurements.
