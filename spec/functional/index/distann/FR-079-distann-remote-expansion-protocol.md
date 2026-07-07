---
id: FR-079
title: Distann Remote Expansion Protocol
type: FR
status: PROPOSED
object: api_endpoint
relationships:
  - target: "ix://agent-ix/ecaz/FR-076"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-079: Distann Remote Expansion Protocol

## Description

Each data node SHALL expose a SQL function `ec_distann_expand_nodes` that
expands a batch of locally-owned graph-node records in one call: it returns
each requested node's exact query distance plus its neighbors' vec_ids with
code-approximated distances, so the coordinator can advance one beam-search
hop round per owned-node batch with a single statement.

## Endpoint

SQL function on every data node, invoked by the coordinator once per owning
node per hop round over the pooled libpq transport:

`ec_distann_expand_nodes(index_regclass regclass, epoch_fingerprint bytea,
query <vector>, vec_ids bigint[], code_threshold real DEFAULT NULL) RETURNS
TABLE (vec_id bigint, exact_dist real, is_tombstone bool, neighbor_vec_ids
bigint[], neighbor_code_dists real[])`

## Inputs

- `index_regclass` — the local ec_distann index
- `epoch_fingerprint` — the coordinator's active epoch identity
- `query` — full-precision query vector
- `vec_ids bigint[]` — locally-owned records to expand (≤ beam width)
- `code_threshold` — optional score floor; neighbors scoring below it MAY be
  omitted

## Outputs

Set of rows: `(vec_id, exact_dist, is_tombstone, neighbor_vec_ids bigint[],
neighbor_code_dists real[])`, one row per requested vec_id.

## Behavior

- The function SHALL validate `epoch_fingerprint` against the node's active
  epoch before any read and raise a retriable epoch-mismatch error on
  disagreement ([FR-082](./FR-082-distann-epoch-lifecycle.md)).
- Expansion SHALL perform, per requested vec_id, exactly one index-record
  read (neighbor scoring SHALL use only the embedded neighbor codes,
  [FR-076](./FR-076-distann-graph-node-record-format.md)) plus exactly one
  co-located heap read of that node's full-precision vector for its exact
  distance. Both reads are node-local — the heap row is co-placed by
  [FR-078](./FR-078-distann-hash-placement.md) — so the call remains one
  network round-trip per node per hop round with no separate rerank
  round-trip.
- Requested vec_ids SHALL resolve to exactly one of four defined outcomes:
  (a) present with its co-placed vector readable (row returned, `exact_dist`
  set, `is_tombstone` reflecting the flag); (b) not owned by this node under
  the epoch's placement → placement error (never an empty result); (c) record
  owned but absent → structural fault error; (d) record present but its
  co-placed vector missing or unreadable (`heap_tid` resolves nothing under
  the epoch) → structural fault error, distinct code from (c). Within a
  published epoch neither records nor the vector tier are physically
  reclaimed ([FR-082](./FR-082-distann-epoch-lifecycle.md)), so cases (c) and
  (d) always indicate corruption or co-placement drift, never a vacuum race
  ([NFR-020](../../../non-functional/NFR-020-distann-fault-behavior.md)); a
  mid-hop failure of either the record read or the vector read maps to the
  corresponding structural fault, non-retriable (distinct from the retriable
  epoch-mismatch of the first bullet).
- `heap_tid` SHALL be interpreted as the epoch-scoped handle to the vec_id's
  frozen co-placed vector ([FR-082](./FR-082-distann-epoch-lifecycle.md)), not
  as a live base-table `ItemPointer` on a data node: a data node is not
  required to host the user base table, only the epoch's vector tier for its
  owned vec_ids. In the single-node degenerate case the handle is the local
  base-table TID under the AM's tombstone/vacuum-consistency handling.
- Exact distances SHALL be computed against the node's co-placed
  full-precision vector (resolved via `heap_tid`,
  [FR-078](./FR-078-distann-hash-placement.md)) — not against any vector
  stored in the index record, which carries none — so the coordinator needs
  no separate rerank round-trip. This is exactly the `ec_diskann`
  coarse-search-then-heap-rerank split, executed node-locally; the
  rerank-fidelity source is table/heap (co-placed, exact), the default and,
  for `ec_diskann`/`ec_distann`, effectively the only mode (ADR-085 D11). For
  a tombstoned record the function MAY omit the vector read and leave
  `exact_dist` unset (NULL), since [FR-081](./FR-081-distann-query-orchestration.md)
  excludes tombstones from results; `is_tombstone` SHALL still be returned so
  the neighbor edges remain usable for traversal continuity.
- `code_threshold` SHALL default to NULL (no pruning), set by the coordinator
  and defaulting to NULL. When set, it is a documented recall-risk
  optimization **outside the scan's correctness guarantees**: it is not a
  fault path, and the [FR-081](./FR-081-distann-query-orchestration.md)-AC-4
  early-exit result-equivalence guarantee holds only at `code_threshold` NULL.
  Because a non-NULL threshold may prune true results, it is never used where
  correctness or the gate is asserted. Gate benchmark runs
  ([NFR-017](../../../non-functional/NFR-017-distann-latency-recall-gate.md))
  SHALL run with `code_threshold` NULL unless the packet pre-registers the
  value and reports its recall effect.
- The response schema of this function is a fixed wire contract independent
  of the record layout (ADR-085 D1): if neighbor codes move from embedded to
  piggybacked, the returned columns do not change.
- The function SHALL execute over the lifted async transport (connection
  pool, batched statements; operational/security posture per
  [NFR-014](../../../non-functional/NFR-014-spire-transport-security-and-operations.md))
  with one call per node per hop round.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-079-AC-1 | Response rows preserve request order and cover every requested vec_id | Test |
| FR-079-AC-2 | Stale epoch_fingerprint yields the retriable epoch-mismatch error, never data | Test |
| FR-079-AC-3 | Non-owned vec_id yields a placement error; owned-but-absent yields a structural fault error; tombstones return normally with the flag set | Test |
| FR-079-AC-4 | Neighbor code distances equal direct QuantCodec scoring of the same codes | Test |
| FR-079-AC-5 | `exact_dist` for each returned vec_id equals the full-precision distance between the query and the node's co-placed heap vector | Test |

## Dependencies

- **Upstream**: [FR-076](./FR-076-distann-graph-node-record-format.md),
  [FR-078](./FR-078-distann-hash-placement.md)
- **Downstream**: [FR-081](./FR-081-distann-query-orchestration.md)
