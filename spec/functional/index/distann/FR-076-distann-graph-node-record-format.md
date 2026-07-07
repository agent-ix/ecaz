---
id: FR-076
title: Distann Graph-Node Record Format and Global Identity
type: FR
status: PROPOSED
object: binary_format
relationships:
  - target: "ix://agent-ix/ecaz/FR-075"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-055"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-076: Distann Graph-Node Record Format and Global Identity

## Description

Each indexed vector SHALL be stored as exactly one graph-node record, keyed
by a global `vec_id`, containing everything a single read needs to expand
the node during beam search: the full-precision vector, the adjacency list,
and a compressed code for each neighbor. Records SHALL be self-describing
and epoch-versioned.

## Layout

```yaml
record: distann_graph_node
version: 1
fields:
  - { name: vec_id, type: u64, rule: global identity per ADR-068 source_identity; unique per logical row }
  - { name: heap_tid, type: item_pointer, rule: owning heap tuple }
  - { name: flags, type: u16, rule: bit0 = tombstone }
  - { name: vector, type: f32[dim], rule: full-precision embedding }
  - { name: neighbor_count, type: u16, rule: "<= graph_degree (R)" }
  - { name: neighbor_vec_ids, type: u64[neighbor_count], rule: adjacency list }
  - { name: neighbor_codes, type: bytes, rule: one neighbor_code_format code per neighbor, fixed stride, scoreable via QuantCodec::score_ip_batch }
```

## Record Fields

| Field | Type | Rule |
|-------|------|------|
| vec_id | u64 | Global identity derived from the ADR-068 source-identity contract; unique per logical row across all nodes and epochs |
| heap_tid | ItemPointer | Owning heap tuple for materialization |
| flags | u16 | Bit 0 = tombstone (deleted, retained until vacuum) |
| vector | f32[dim] | Full-precision embedding |
| neighbor_count | u16 | ≤ `graph_degree` (R) |
| neighbor_vec_ids | u64[neighbor_count] | Adjacency list |
| neighbor_codes | byte block | One `neighbor_code_format` code per neighbor, fixed stride, scoreable via `QuantCodec::score_ip_batch` without any further read |

## Behavior

- The record format SHALL carry a format-version tag in the index metadata
  page following the `VamanaMetadataPage` convention; version bumps follow
  [NFR-016](../../../non-functional/NFR-016-on-disk-format-evolution-discipline.md)
  (research posture: rebuild, no migration).
- `vec_id` SHALL be stable across index rebuilds for the same logical row
  (derived from `source_identity`), so cross-epoch and cross-node references
  never alias distinct rows. The derivation (hash64 with collision handling
  vs dense per-epoch assignment) is fixed by ADR-085 decision D6.
- Expanding a record SHALL require exactly one record read: scoring all its
  neighbors uses the embedded codes, never a secondary lookup.
- Tombstoned records SHALL remain readable (for graph traversal continuity)
  but SHALL be excluded from result sets.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-076-CON-1 | Total on-disk bytes SHALL stay within the NFR-018 space-amplification budget | Storage | Benchmark storage step |
| FR-076-CON-2 | neighbor_count SHALL NOT exceed the `graph_degree` reloption | Integrity | Unit test + build assertion |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-076-AC-1 | Record round-trip (encode → write → read → decode) preserves vector, adjacency, and neighbor codes byte-exactly | Test |
| FR-076-AC-2 | Two rebuilds of the same corpus assign identical vec_ids to identical source rows | Test |
| FR-076-AC-3 | Scoring a node's neighbors after one record read matches direct codec scoring of the neighbors' vectors | Test |
| FR-076-AC-4 | Tombstoned records are traversable but never returned | Test |

## Dependencies

- **Upstream**: [FR-075](./FR-075-ec-distann-access-method-surface.md); the
  ADR-068 source-identity contract; ADR-085 decisions D1 (code duplication),
  D6 (vec_id derivation), D7 (codec choice)
- **Downstream**: [FR-077](./FR-077-distann-sharded-build-and-stitch.md),
  [FR-079](./FR-079-distann-remote-expansion-protocol.md)
