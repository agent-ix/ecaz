---
id: FR-078
title: Distann Hash Placement and Placement Directory
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-076"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-055"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-077"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-078: Distann Hash Placement and Placement Directory

## Description

Graph-node records SHALL be distributed across nodes by a deterministic hash
of vec_id, with each record's co-placed heap row (its full-precision vector,
the rerank tier of [FR-076](./FR-076-distann-graph-node-record-format.md))
landing on the same hash-owned node so exact rerank
([FR-079](./FR-079-distann-remote-expansion-protocol.md)) is always a
node-local read. Placement SHALL affect load balance only — never recall or
graph structure — and SHALL be resolvable by any coordinator without a
per-record directory lookup.

## Behavior

- The owning node of a record SHALL be `hash(vec_id) mod node_count` under
  the epoch's registered node roster.
- The placement function SHALL map a record's full-precision heap row
  (referenced by `heap_tid`) to the same owning node as the record itself,
  co-placing the heap tier by the identical `hash(vec_id)` so no record's
  exact-rerank vector lives on a different node than the record. In the
  degenerate single-node deployment (M0) this is satisfied by the index and
  its base table sharing one instance; in a multi-node deployment the
  build→publish hand-off co-locates the vector with the record (below).
- The epoch manifest SHALL fix the hash function version and roster ordering
  so every participant computes identical placement.
- An epoch-stamped placement directory (adapted — not shared — from
  `SpirePlacementDirectory`, whose contract stays owned by the SPIRE specs)
  SHALL carry topology metadata only: node roster, roster ordering, hash
  function version, and per-node record counts. It SHALL NOT store
  per-record entries.
- The build→publish hand-off SHALL be owned by the **coordinator's epoch
  build pipeline** (the same component that runs FR-077 and FR-082 assembly):
  after the FR-077 stitch emits records, it SHALL write each record **and its
  full-precision vector (heap row)** to the same hash-owned node — issuing the
  per-node writes over the lifted transport ([NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md))
  — and then publish the epoch ([FR-082](./FR-082-distann-epoch-lifecycle.md));
  no other component moves records or vectors. The vector is stored once, on
  the owning node, and is never duplicated into the index record (ADR-085
  decision D11).
- When the roster changes, a new epoch SHALL be built and published; queries
  against the old epoch continue against the old roster until retirement
  ([FR-082](./FR-082-distann-epoch-lifecycle.md)).
- The coordinator SHALL group any set of vec_ids by owning node in O(set
  size) using only the manifest.
- At expansion time, if a record's `heap_tid` does not resolve to a
  node-local vector under the epoch (co-placement drift — record and vector
  diverged across nodes), the owning node SHALL raise the structural fault of
  [FR-079](./FR-079-distann-remote-expansion-protocol.md) case (d), never a
  silent skip or a remote fetch. Co-placement is thus enforced at runtime,
  not only asserted at build time (AC-4).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-078-AC-1 | Placement of any vec_id is identical when computed on coordinator and on every data node | Test |
| FR-078-AC-2 | Record counts per node are within 10% of uniform at 100k across 3 nodes | Analysis (build stats) |
| FR-078-AC-3 | A roster change never alters placement within a published epoch | Test |
| FR-078-AC-4 | For every vec_id, its graph record and its full-precision heap row resolve to the same owning node under the epoch roster | Test |

## Dependencies

- **Upstream**: [FR-076](./FR-076-distann-graph-node-record-format.md)
- **Downstream**: [FR-079](./FR-079-distann-remote-expansion-protocol.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md)
