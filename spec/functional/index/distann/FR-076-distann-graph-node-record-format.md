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
---
# FR-076: Distann Graph-Node Record Format and Global Identity

## Description

Each indexed vector SHALL be stored as exactly one graph-node record, keyed
by a global `vec_id`, containing everything a single read needs to score and
expand the node during beam search: a coarse search code, the adjacency list,
and a compressed code for each neighbor. The record SHALL NOT store the
full-precision vector; the exact distance of an expanded node is computed
from its co-placed heap row ([FR-078](./FR-078-distann-hash-placement.md),
[FR-079](./FR-079-distann-remote-expansion-protocol.md)), so the corpus
vectors live once in the heap tier and are never duplicated into the index
(ADR-085 decision D11). Records SHALL be self-describing and epoch-versioned.

This is the `ec_diskann` record shape (coarse code + adjacency; full vector
in the heap), sharded: coarse code drives beam ordering, the co-placed heap
row supplies exact rerank.

## Layout

```yaml
record: distann_graph_node
version: 1
fields:
  - { name: vec_id, type: u64, rule: global identity per ADR-068 source_identity; unique per logical row }
  - { name: heap_tid, type: item_pointer, rule: owning heap tuple; resolves the co-placed full-precision vector for exact rerank (FR-079) and final materialization }
  - { name: flags, type: u16, rule: bit0 = tombstone }
  - { name: search_code, type: bytes, rule: one neighbor_code_format code for this node's own vector, fixed stride; used to score the node when it enters the beam }
  - { name: neighbor_count, type: u16, rule: "<= graph_degree (R)" }
  - { name: neighbor_vec_ids, type: u64[neighbor_count], rule: adjacency list }
  - { name: neighbor_codes, type: bytes, rule: one neighbor_code_format code per neighbor, fixed stride, scoreable via QuantCodec::score_ip_batch }
```

## Record Fields

| Field | Type | Rule |
|-------|------|------|
| vec_id | u64 | Global identity derived from the ADR-068 source-identity contract; unique per logical row across all nodes and epochs |
| heap_tid | ItemPointer | Owning heap tuple; the co-placed heap row it resolves ([FR-078](./FR-078-distann-hash-placement.md)) is the single source of the node's full-precision vector for exact rerank ([FR-079](./FR-079-distann-remote-expansion-protocol.md)) and for final materialization |
| flags | u16 | Bit 0 = tombstone (deleted, retained until vacuum) |
| search_code | byte block | One `neighbor_code_format` code for this node's own vector, fixed stride; scores the node when it enters the beam without a heap read |
| neighbor_count | u16 | ≤ `graph_degree` (R) |
| neighbor_vec_ids | u64[neighbor_count] | Adjacency list |
| neighbor_codes | byte block | One `neighbor_code_format` code per neighbor, fixed stride, scoreable via `QuantCodec::score_ip_batch` without any further read |

## Behavior

- The record format SHALL carry a format-version tag in the index metadata
  page following the `VamanaMetadataPage` convention; version bumps follow
  [NFR-016](../../../non-functional/NFR-016-on-disk-format-evolution-discipline.md)
  (research posture: rebuild, no migration).
- `vec_id` SHALL be stable across index rebuilds for the same logical row,
  derived from `source_identity` per the ADR-063 source-identity provider
  contract (reached via ADR-068's distributed topology), so cross-epoch and
  cross-node references never alias distinct rows. The derivation (hash64
  with collision handling vs dense per-epoch assignment) is fixed by
  ADR-085 decision D6. Placement
  ([FR-078](./FR-078-distann-hash-placement.md)) consumes this identity.
- Expanding a record SHALL require exactly one index-record read to score all
  its neighbors: neighbor scoring uses the embedded codes, never a secondary
  lookup. The expanded node's exact distance SHALL come from a single read of
  its co-placed heap row (via `heap_tid`), not from any vector stored in the
  record ([FR-079](./FR-079-distann-remote-expansion-protocol.md)); both reads
  are node-local (the heap row is co-placed by [FR-078](./FR-078-distann-hash-placement.md)),
  so expansion adds no network round-trip.
- The record SHALL NOT carry the full-precision vector. The single
  authoritative copy of each vector lives in the co-placed heap tier; the
  index stores only codes and adjacency. This removes exactly 1.0× the raw
  vector bytes from per-record amplification versus an inline-vector layout
  (ADR-085 decisions D1, D11; [NFR-018](../../../non-functional/NFR-018-distann-space-amplification.md)).
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
| FR-076-AC-1 | Record round-trip (encode → write → read → decode) preserves search code, adjacency, and neighbor codes byte-exactly | Test |
| FR-076-AC-2 | Two rebuilds of the same corpus assign identical vec_ids to identical source rows | Test |
| FR-076-AC-3 | Scoring a node's neighbors after one record read matches direct codec scoring of the neighbors' vectors | Test |
| FR-076-AC-4 | Tombstoned records are traversable but never returned | Test |
| FR-076-AC-5 | The encoded record layout contains no full-precision vector field (structural inspection of the decoded record) | Test |
| FR-076-AC-6 | At fixed R and `neighbor_code_format`, encoded record byte size is independent of vector dimension | Test |

## Dependencies

- **Upstream**: [FR-075](./FR-075-ec-distann-access-method-surface.md); the
  ADR-068 source-identity contract; ADR-085 decisions D1 (code duplication),
  D6 (vec_id derivation), D7 (codec choice), D11 (co-placed heap rerank)
- **Downstream**: [FR-077](./FR-077-distann-sharded-build-and-stitch.md),
  [FR-078](./FR-078-distann-hash-placement.md) (co-places the heap row),
  [FR-079](./FR-079-distann-remote-expansion-protocol.md)
