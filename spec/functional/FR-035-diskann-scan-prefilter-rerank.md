---
id: FR-035
title: DiskANN Scan, Prefilter, and Rerank
type: functional-requirement
artifact_type: FR
status: IMPLEMENTED
object_type: process
relationships:
  - target: "ix://agent-ix/ecaz/US-014"
    type: "implements"
    cardinality: "N:1"
---
# FR-035: DiskANN Scan, Prefilter, and Rerank

## Requirement

`ec_diskann` SHALL implement ordered scan over the persisted Vamana graph using a configurable traversal prefilter and heap rerank.

## Behavior

1. Scan breadth SHALL resolve from relation `list_size` unless `ec_diskann.list_size` session override is set.
2. `ec_diskann.prefilter_kind` SHALL accept `auto`, `binary_sidecar`, and `grouped_pq`.
3. `auto` SHALL use persisted binary sidecars when available and fall back to grouped-PQ behavior when required.
4. `rerank_budget` SHALL bound final exact heap rerank before executor LIMIT truncation.
5. Costing SHALL model DiskANN scan behavior without replacing HNSW as the default guidance.

## Acceptance Criteria

### FR-035-AC-1

`ORDER BY embedding <#> query LIMIT k` returns ordered results through `ec_diskann`.

### FR-035-AC-2

Session `ec_diskann.list_size` changes the effective scan breadth.

### FR-035-AC-3

The binary sidecar prefilter path and grouped-PQ fallback are selectable through `ec_diskann.prefilter_kind`.
