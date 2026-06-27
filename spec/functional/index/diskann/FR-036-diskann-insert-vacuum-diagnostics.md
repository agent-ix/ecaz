---
id: FR-036
title: DiskANN Insert, Vacuum, and Diagnostics
type: FR
status: IMPLEMENTED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/US-014"
    type: "implements"
    cardinality: "N:1"
---
# FR-036: DiskANN Insert, Vacuum, and Diagnostics

## Description

`ec_diskann` SHALL support live insert, duplicate handling, vacuum repair, and graph diagnostics for the persisted Vamana format.

## Behavior

1. Live insert SHALL add a new node or duplicate overflow entry according to persisted duplicate state.
2. Insert SHALL maintain Vamana lock ordering and graph-degree constraints.
3. Vacuum SHALL remove dead primary heap TIDs, promote duplicate overflow entries when possible, tombstone dead nodes, repair neighbor slots, and mark medoid refresh when needed.
4. Diagnostics SHALL expose graph summary state for review packets and tuning.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-036-AC-1 | Rows inserted after DiskANN index creation are reachable through the index | Test |
| FR-036-AC-2 | DELETE plus VACUUM removes dead DiskANN entries and repairs affected neighbor slots | Test |
| FR-036-AC-3 | DiskANN diagnostics expose graph summary metadata without mutating the index | Test |

### FR-036-AC-1

Rows inserted after DiskANN index creation are reachable through the index.

### FR-036-AC-2

DELETE plus VACUUM removes dead DiskANN entries and repairs affected neighbor slots.

### FR-036-AC-3

DiskANN diagnostics expose graph summary metadata without mutating the index.

## Workflow

```mermaid
flowchart TD
    subgraph Insert["aminsert"]
        IA["Derive payload from persisted codebooks (SRHT + grouped-PQ)"] --> IB{"empty index?"}
        IB -->|"yes"| IC["Bootstrap first node"]
        IB -->|"no"| ID{"exact duplicate of an existing node?"}
        ID -->|"yes"| IE["Bind heap TID to that node's overflow chain"]
        ID -->|"no"| IF["Greedy-search + alpha-prune forward neighbors (<= R)"]
        IF --> IG["Append new node tuple"]
        IG --> IH["Add backlinks where neighbor slots free, else re-prune (keep degree <= R)"]
    end
    subgraph Vacuum["ambulkdelete + amvacuumcleanup"]
        VA["Strip dead primary heap TIDs (promote live overflow when possible)"] --> VB["Repair neighbor lists (drop dead TIDs, compact, pad INVALID)"]
        VB --> VC["Tombstone fully-dead nodes"]
        VC --> VD["Flag needs_medoid_refresh when medoid affected"]
    end
    subgraph Diag["ec_diskann_index_admin_snapshot(regclass)"]
        DA["Materialize persisted graph (read-only)"] --> DB["Report node count, degree stats, medoid, inserted_since_rebuild"]
    end
```

## Dependencies

- **Upstream**: US-014 (implements relationship)
- **Downstream**: none identified
