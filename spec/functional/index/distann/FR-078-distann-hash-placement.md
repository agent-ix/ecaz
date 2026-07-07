---
id: FR-078
title: Distann Hash Placement and Placement Directory
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-076"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-078: Distann Hash Placement and Placement Directory

## Description

Graph-node records SHALL be distributed across nodes by a deterministic hash
of vec_id. Placement SHALL affect load balance only — never recall or graph
structure — and SHALL be resolvable by any coordinator without a per-record
directory lookup.

## Behavior

- The owning node of a record SHALL be `hash(vec_id) mod node_count` under
  the epoch's registered node roster.
- The epoch manifest SHALL fix the hash function version and roster ordering
  so every participant computes identical placement.
- An epoch-stamped placement directory (adapted from
  `SpirePlacementDirectory`) SHALL carry topology metadata only: node
  roster, roster ordering, hash function version, and per-node record
  counts. It SHALL NOT store per-record entries.
- When the roster changes, a new epoch SHALL be built and published; queries
  against the old epoch continue against the old roster until retirement
  ([FR-082](./FR-082-distann-epoch-lifecycle.md)).
- The coordinator SHALL group any set of vec_ids by owning node in O(set
  size) using only the manifest.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-078-AC-1 | Placement of any vec_id is identical when computed on coordinator and on every data node | Test |
| FR-078-AC-2 | Record counts per node are within 10% of uniform at 100k across 3 nodes | Analysis (build stats) |
| FR-078-AC-3 | A roster change never alters placement within a published epoch | Test |

## Dependencies

- **Upstream**: [FR-076](./FR-076-distann-graph-node-record-format.md)
- **Downstream**: [FR-079](./FR-079-distann-remote-expansion-protocol.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md)
