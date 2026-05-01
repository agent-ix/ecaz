---
id: FR-031
title: IVF Build and Storage
type: functional-requirement
artifact_type: FR
status: IMPLEMENTED
object_type: process
relationships:
  - target: "ix://agent-ix/tqvector/US-013"
    type: "implements"
    cardinality: "N:1"
---
# FR-031: IVF Build and Storage

## Requirement

`ec_ivf` SHALL implement a PostgreSQL index access method that trains centroids, assigns heap rows to posting lists, and persists AM-owned metadata and posting-list pages.

## Behavior

1. `ec_ivf` SHALL support `ecvector_ip_ops` and `tqvector_ip_ops`.
2. Build reloptions SHALL include `nlists`, `nprobe`, `rerank_width`, `training_sample_rows`, `seed`, `pq_group_size`, `posting_slack_percent`, `storage_format`, and `rerank`.
3. `storage_format` SHALL accept `auto`, `turboquant`, `pq_fastscan`, and `rabitq`.
4. `rerank` SHALL accept `auto`, `off`, and `heap_f32`; `source_column` SHALL be rejected until implemented.
5. Training and assignment SHALL be deterministic for the same data and seed.
6. Posting slack pages SHALL be reserved when configured for churn reuse.

## Acceptance Criteria

### FR-031-AC-1

`CREATE INDEX ... USING ec_ivf` produces readable IVF metadata with centroid/list counts and storage-format metadata.

### FR-031-AC-2

Invalid reloption values raise ERROR during index creation.

### FR-031-AC-3

`rerank = 'source_column'` raises a clear unsupported-mode ERROR.
