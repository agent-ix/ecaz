---
id: ADR-068
title: "SPIRE Distributed Table Topology"
status: PROPOSED
impact: Defines coordinator and remote-node table shapes, placement metadata,
  and the read-path data flow under ADR-067 CustomScan. Affects Task 30
  Phase 11 Stage D scope.
date: 2026-05-10
---
# ADR-068: SPIRE Distributed Table Topology

## Status

Proposed.

## Related

- ADR-067 selects CustomScan as the distributed scan integration point.
  This ADR describes the data layout that CustomScan operates over.
- ADR-049 governs SPIRE partition-object storage; this ADR is the
  distributed extension of that storage layout.
- ADR-063 (source identity) defines the global vector identity that ties
  rows on remote shards back to logical entity identity.
- ADR-069 defines the write-path scope and explicit deferrals.

## Context

ADR-067 establishes that the production distributed scan is a CustomScan
node, not an index AM cursor. That removes the requirement for a same-
relation coordinator mirror. The remaining design question is: where do
rows actually live, and what does the coordinator's table look like?

## Decision

SPIRE v1 distributed deployments have the following topology:

### Coordinator role

The coordinator hosts:

- The **logical relation** for the distributed table (e.g., `documents`).
  The coordinator's local heap for this relation MAY be empty
  (router-only coordinator) or MAY hold rows assigned to a local shard
  (router-plus-shard coordinator).
- The **SPIRE routing index** built over the logical relation. This index
  holds routing centroids and placement metadata: centroid → node_id
  mappings, remote-node descriptors, served-epoch state.
- **Remote-node descriptors** registered via the existing
  `ec_spire_register_remote_node_descriptor` plumbing.

The coordinator does **not** hold mirrors of rows whose centroid is
assigned to a remote node.

### Remote-node role

Each remote node hosts:

- A **shard table** with the same column shape as the coordinator's
  logical relation. The shard's primary key SHALL be globally unique
  across all shards (UUID, coordinator-allocated sequence, or
  ADR-063-style source identity column).
- A **local SPIRE index** built over the shard's rows. The remote's
  SPIRE index uses the same `nlists`, `nprobe`, source-identity
  configuration, and quantizer profile as the coordinator's index, so
  remote scoring is identical to local scoring.
- **Remote endpoint registration** via the existing
  `ec_spire_announce_remote_index` plumbing.

The remote is operationally a normal PostgreSQL instance with the `ecaz`
extension installed. From the remote's perspective there is no
distinction between serving SPIRE queries as a shard versus running
SPIRE locally.

### Placement

Placement maps centroid identity to node identity:

- The coordinator's routing centroids are trained from a sample drawn
  across all shards or seeded from an initial bulk-load distribution.
- A placement policy (default: `centroid_assigned`) assigns each top-
  level centroid to exactly one node. Centroid-to-node assignment is
  stored in the coordinator's placement metadata and published as part
  of the SPIRE epoch.
- Boundary replicas (ADR-049) may be hosted on multiple nodes for
  routing accuracy near centroid borders, with replica dedupe applied at
  merge time per ADR-055.

A row's home node is determined at insert time by classifying its
embedding against the active routing centroids. The placement is opaque
to the application; the application sees one logical table.

### Read flow

Read flow under ADR-067 CustomScan:

1. Application issues
   `SELECT cols FROM documents ORDER BY embedding <-> $1 LIMIT k` at
   the coordinator.
2. Planner registers and selects the `EcSpireDistributedScan` CustomScan
   path for the `documents` relation because its `ec_spire` index has
   active remote placements.
3. CustomScan `Exec`:
   a. Loads the active epoch routing centroids and placement metadata
      from the coordinator's SPIRE index.
   b. Runs the SPIRE routing traversal to identify the leaf PIDs whose
      centroids are nearest the query vector.
   c. Groups selected PIDs by `(node_id, local_store_id)` per the
      placement metadata.
   d. For local-shard PIDs (if the coordinator is a router-plus-shard
      coordinator), scores locally through the existing scan path.
   e. For remote PIDs, dispatches a remote search request to each
      target node through the production transport adapter (Stage C
      executor state machine).
   f. Remote nodes score against their local SPIRE index, resolve heap
      visibility on the origin node (ADR-059), and return ordered
      compact tuple payloads (extended endpoint contract; see below).
   g. CustomScan merges the local and remote candidate streams with
      deterministic tie-breaks, applies bounded heap rerank, and
      returns the top-k tuples directly to the PostgreSQL executor.
4. The application receives a result set with rows from any shard,
   sorted by vector distance, with no awareness of distribution.

### Endpoint contract extension

The Stage B remote endpoint contract returns a candidate envelope with
identity, score, heap coordinates, and diagnostics. Under ADR-068 the
endpoint additionally returns the **tuple columns** the coordinator
needs to satisfy the application's `SELECT` projection.

The endpoint contract surface adds an optional `tuple_payload` payload
keyed by `(node_id, vec_id)`. The coordinator declares the required
columns at request time; the remote materializes those columns from its
local heap after origin-node visibility resolution succeeds, then
returns them on the response.

The 18-column envelope from Stage B is unchanged; the tuple payload is
an attached side-channel rather than a column-shape change. Existing
identity, fingerprint, version, and status fields remain.

## Required Invariants

- The coordinator's `documents` table (or equivalent logical relation)
  is the authoritative target of all DDL — `CREATE INDEX`, `ALTER
  TABLE`, etc. — for the distributed logical entity. Remote shards
  follow.
- Remote shard tables MUST have the same column types and constraints as
  the coordinator's logical relation. Schema drift between coordinator
  and remote is a fault matrix case (already covered by Stage B endpoint
  identity / version-skew gates).
- A row's home node MUST be deterministic from its embedding given the
  active epoch's routing centroids. If centroids change (retraining),
  some rows may need to migrate; migration is a Phase 12+ scope
  question.
- The CustomScan path MUST NOT depend on the coordinator's local heap
  containing the row data for any remote-origin candidate. The catalog
  table from ADR-065 and the mirror sync from ADR-066 are no longer
  required for read correctness.

## Consequences

- A distributed deployment has N+1 PostgreSQL instances: 1 coordinator
  plus N remote shard hosts. The coordinator may also be a shard host
  for hybrid deployments.
- The user experience for vector-ordered reads is identical to a single-
  machine SPIRE deployment: write `SELECT cols FROM table ORDER BY
  embedding <-> $1 LIMIT k`, get rows back.
- Non-vector reads (e.g., `SELECT cols FROM documents WHERE id = $1`)
  against the coordinator return only rows that physically live in the
  coordinator's local heap. If the coordinator is router-only, the
  result set is empty. This is a write-side concern documented in
  ADR-069.
- Bulk-load workflows can write directly to the appropriate remote per
  ADR-069 Option II, bypassing coordinator write pressure entirely.

## Operator-facing setup sketch

This is illustrative, not normative. Final SQL surface lands in the
implementation packet.

```sql
-- On each remote node (one-time setup per node):
CREATE EXTENSION ecaz;
CREATE TABLE documents (
  id              bigint PRIMARY KEY,
  title           text,
  body            text,
  embedding       real[],
  source_identity uuid NOT NULL UNIQUE
);
CREATE INDEX documents_embedding_idx ON documents
  USING ec_spire (embedding)
  INCLUDE (source_identity)
  WITH (
    nlists = 256,
    rerank_width = 50,
    source_identity = 'include',
    storage_format = 'rabitq'
  );
SELECT ec_spire_announce_remote_index(
  index_oid    => 'documents_embedding_idx'::regclass,
  served_epoch => 1
);

-- On the coordinator (one-time setup):
CREATE EXTENSION ecaz;
CREATE TABLE documents (
  id              bigint PRIMARY KEY,
  title           text,
  body            text,
  embedding       real[],
  source_identity uuid NOT NULL UNIQUE
) WITH (ec_spire_distributed = true);

SELECT ec_spire_register_remote_node(
  node_id              => 2,
  conninfo_secret_name => 'remote_node_2_conninfo',
  remote_relation      => 'documents'
);
SELECT ec_spire_register_remote_node(
  node_id              => 3,
  conninfo_secret_name => 'remote_node_3_conninfo',
  remote_relation      => 'documents'
);

CREATE INDEX documents_embedding_idx ON documents
  USING ec_spire (embedding)
  INCLUDE (source_identity)
  WITH (
    nlists              = 256,
    nprobe              = 8,
    source_identity     = 'include',
    distribution        = 'centroid_assigned',
    remote_nodes        = '{2,3}'
  );
```

After setup, the application issues `SELECT ... ORDER BY embedding <-> $1
LIMIT k` against the coordinator and the CustomScan handles fanout.

## Open Questions

- Should the coordinator's logical relation be declared via a new
  reloption (`ec_spire_distributed = true`) or via a separate registration
  function? Implementation-detail; sketch shows reloption.
- How to handle centroid retraining when row distribution drifts. Likely
  a Phase 12+ scope.
- How to handle a new remote node joining an existing deployment (rebalance
  semantics). Likely a Phase 12+ scope.
