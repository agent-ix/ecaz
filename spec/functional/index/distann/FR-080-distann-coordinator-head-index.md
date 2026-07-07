---
id: FR-080
title: Distann Coordinator Head Index
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-077"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-080: Distann Coordinator Head Index

## Description

The coordinator SHALL maintain an in-memory head index — a Vamana graph over
a bounded sample of the global graph's entry region — so a query's first
hops execute locally with zero network round trips and hop rounds start deep
in the correct region of the global graph.

## Behavior

- At build time, the pipeline SHALL collect a breadth-first sample of up to
  `head_index_cap` (C) vectors from the global graph's entry region (union
  across build shards' top layers, guaranteeing every shard's region is
  reachable) and persist the sample with the epoch.
- The coordinator SHALL construct the in-memory head index from the persisted
  sample on first use per epoch (reusing the in-memory Vamana builder used
  by the SPIRE top-graph) and cache it keyed on `(index_oid, epoch)`.
- A query SHALL search the head index first; its best results seed the hop
  round frontier of [FR-081](./FR-081-distann-query-orchestration.md).
- Head-index construction SHALL be deterministic under a fixed seed.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-080-CON-1 | Head-index memory SHALL be bounded by C × (vector bytes + graph overhead); C is a reloption with a documented default | Memory | Analysis + unit test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-080-AC-1 | Head-index search returns entry candidates without any remote call | Test |
| FR-080-AC-2 | Construction is deterministic for a fixed seed and epoch | Test |
| FR-080-AC-3 | Every build shard's region is reachable from the head sample | Test (property/BFS) |
| FR-080-AC-4 | Recall sensitivity to C is measured and recorded at M0 (informs the default) | Analysis (bench) |

## Dependencies

- **Upstream**: [FR-077](./FR-077-distann-sharded-build-and-stitch.md);
  ADR-085 decision D3 (C policy)
- **Downstream**: [FR-081](./FR-081-distann-query-orchestration.md)
