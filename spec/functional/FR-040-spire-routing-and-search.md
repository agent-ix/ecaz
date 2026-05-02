---
id: FR-040
title: SPIRE Routing and Search Execution
type: functional-requirement
artifact_type: FR
status: DRAFT
object_type: process
relationships:
  - target: "ix://agent-ix/tqvector/US-017"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/FR-038"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-040: SPIRE Routing and Search Execution

## Requirement

`ec_spire` SHALL route query vectors through SPIRE-owned root and hierarchy metadata, fetch selected partition objects by PID, score candidates near the partition data, deduplicate boundary replicas, and return ordered results through PostgreSQL execution.

## Behavior

1. PostgreSQL's planner SHALL only decide whether to use the SPIRE access path.
2. SPIRE SHALL choose PIDs from the query vector using root graph or centroid routing plus hierarchy metadata.
3. Single-level v1 MAY use a flat centroid router before the root graph lands.
4. Recursive SPIRE SHALL route top-down from root graph to internal partition objects to leaf partition objects.
5. Leaf scoring SHALL use the selected quantizer/profile payload stored in assignment/posting rows.
6. Boundary replicas SHALL be deduplicated by stable `vec_id` before final top-k emission.
7. Local heap visibility SHALL remain PostgreSQL executor responsibility for local rows.
8. If a candidate carries a heap TID that no longer identifies the indexed row version, the scan SHALL suppress or repair that candidate through the update/vacuum policy instead of emitting a wrong tuple.

## Search Sequence

```mermaid
sequenceDiagram
    participant SQL as SQL executor
    participant AM as ec_spire AM
    participant Root as Root graph / centroid router
    participant Place as Placement map
    participant Store as Partition stores
    participant Heap as Heap executor

    SQL->>AM: amrescan(query vector, k)
    AM->>Root: choose top PIDs for active epoch
    Root-->>AM: selected PIDs
    AM->>Place: group PIDs by local store
    Place-->>AM: store batches
    AM->>Store: fetch and score leaf objects
    Store-->>AM: candidate vec_id, heap_tid, score
    AM->>AM: merge and dedupe by vec_id
    AM-->>SQL: ordered heap TIDs
    SQL->>Heap: visibility check and row fetch
```

## Routing Topology

```mermaid
flowchart TD
    Q["query vector"]
    G["root graph / top centroid router"]
    L2["internal partition objects\nlevel 2"]
    L1["internal partition objects\nlevel 1"]
    Leaf["leaf partition objects\nassignment rows"]
    K["merged top-k"]

    Q --> G
    G --> L2
    L2 --> L1
    L1 --> Leaf
    Leaf --> K
```

## Acceptance Criteria

### FR-040-AC-1

Single-level SPIRE can route to leaf PIDs, score candidates, and return ordered local heap TIDs.

### FR-040-AC-2

Recursive SPIRE can route through at least two hierarchy levels before leaf scoring.

### FR-040-AC-3

Boundary replica deduplication keeps the best candidate for a `vec_id` and exposes diagnostics for duplicate candidates suppressed.
