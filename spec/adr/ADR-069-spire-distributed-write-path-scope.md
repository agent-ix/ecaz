---
id: ADR-069
title: "SPIRE Distributed Write Path Scope"
status: PROPOSED
impact: Declares the v1 distributed INSERT/UPDATE/DELETE contract under
  ADR-067 CustomScan and ADR-068 distributed table topology. Bulk load and
  cross-shard embedding moves are deferred.
date: 2026-05-10
---
# ADR-069: SPIRE Distributed Write Path Scope

## Status

Proposed.

## Related

- ADR-067 selects CustomScan as the distributed read path.
- ADR-068 defines the coordinator + remote shard topology.
- ADR-063 source identity is required for stable row identification across
  shards.
- ADR-055 vector identity contract governs global identity bytes.

## Context

ADR-067 and ADR-068 deliver a working distributed read path: rows live on
remote shards, the coordinator hosts routing centroids and placement
metadata, and `SELECT ... ORDER BY embedding <-> $1 LIMIT k` against the
coordinator's logical relation fans out via CustomScan and returns rows
directly.

For a useful v1 production system the corresponding write path must also
work cleanly. Applications need to INSERT, UPDATE, and DELETE rows
without managing shard placement themselves. The coordinator handles
distribution transparently. Bulk-load workflows that bypass the
coordinator for throughput are a separate task and a separate operational
mode.

## Decision

The v1 SPIRE distributed table write contract is **coordinator-routed
writes** for normal application workloads, with explicit bulk-load
escape hatches available for high-throughput ingestion.

### Coordinator-routed INSERT

Applications INSERT against the coordinator's logical relation:

```sql
INSERT INTO documents (id, title, body, embedding, source_identity)
VALUES ($1, $2, $3, $4, $5);
```

The coordinator:

1. Classifies the embedding against the active SPIRE routing centroids
   to determine the target `node_id`.
2. Opens a libpq dispatch to the target remote using the
   transport adapter (the same one Stage C builds for reads).
3. Forwards the INSERT to the target remote, which executes it against
   its local shard table and SPIRE index (using the existing
   `aminsert` code path).
4. Records the `(id, node_id, served_epoch)` mapping in a coordinator-
   local **placement directory** (described below) so subsequent
   UPDATE/DELETE/PK reads can find the row.
5. Returns the INSERT result (rowcount, any RETURNING data) to the
   application.

The INSERT is atomic from the application's perspective:
- If the remote INSERT succeeds and the coordinator placement directory
  update succeeds, the operation commits.
- If either fails, the operation rolls back. The dispatch transaction on
  the remote either never commits or is rolled back by the coordinator
  on failure.
- The coordinator uses **two-phase commit (PREPARE TRANSACTION)** on the
  remote dispatch to coordinate atomicity. The remote prepares the
  transaction; the coordinator commits its placement directory update
  and then commits the remote prepared transaction. If anything between
  prepare and commit fails, the operator-resolution recovery path is
  the standard PostgreSQL prepared-transaction recovery flow.

### Coordinator-routed UPDATE (non-embedding columns)

Applications UPDATE non-embedding columns against the coordinator's
logical relation:

```sql
UPDATE documents SET title = $1 WHERE id = $2;
```

The coordinator:

1. Looks up `node_id` for `id` in the placement directory.
2. Forwards the UPDATE to the owning remote.
3. Remote executes the UPDATE against its local heap (SPIRE index sees
   no change since the embedding is unchanged).
4. Coordinator returns the UPDATE result to the application.

No two-phase commit is required because no coordinator-side state changes
on a non-embedding UPDATE.

### Coordinator-routed DELETE

```sql
DELETE FROM documents WHERE id = $1;
```

The coordinator:

1. Looks up `node_id` in the placement directory.
2. Forwards the DELETE to the owning remote (within a prepared
   transaction).
3. On remote success, removes the row from the placement directory.
4. Commits both the placement-directory deletion and the remote prepared
   transaction.

Two-phase commit applies for the same reason as INSERT: the placement
directory is coordinator-side state that must stay consistent with the
remote.

### UPDATE of the embedding column

An UPDATE that changes the embedding column may change the row's home
node. v1 chooses the simplest correctness contract:

**v1 rejects embedding-changing UPDATE with a clear error:**

```text
ERROR: ec_spire_distributed: UPDATE of indexed embedding column is not
supported on a distributed ec_spire table. Use DELETE + INSERT.
HINT: Cross-shard atomic moves will be available in a future release.
```

Applications that need to change an embedding perform an explicit
DELETE followed by INSERT in their own transaction (or transactions).
This is operationally identical to how Citus handles updates to
distribution-key columns.

A future ADR may add atomic cross-shard UPDATE-as-move. v1 keeps
the contract narrow.

### Bulk-load escape hatch (separate task)

For bulk-load workflows where coordinator-routed INSERT is a throughput
bottleneck, applications may write **directly to remote shards** using
a coordinator-provided classification helper:

```sql
-- Coordinator-side classification:
SELECT ec_spire_classify_centroid(
  embedding   => $1::real[],
  index_oid   => 'documents_embedding_idx'::regclass
);
-- Returns (node_id, centroid_id, epoch).
```

The bulk-load tool batches rows by `node_id`, opens parallel libpq
connections to each remote, and INSERTs in bulk. After the bulk load
completes, the tool calls a coordinator primitive to register the
placement entries in batch:

```sql
SELECT ec_spire_register_placement_batch(
  index_oid => 'documents_embedding_idx'::regclass,
  entries   => $1::ec_spire_placement_entry[]
);
```

The exact bulk-load CLI surface, batching primitives, and
parallel-ingest mechanism are **out of scope for v1** and will be
delivered in a separate task with its own packets. The coordinator-side
classification helper and batch-register primitive are the only v1
contracts.

`ec_spire_register_placement_batch` runs inside the caller's
transaction. A primary-key conflict, catalog constraint violation, or
NULL element in the `entries` array aborts the whole batch; callers that
need partial recovery must split work into smaller transactions. The v1
`ec_spire_placement_entry` composite field order is frozen as
`(pk_value, node_id, centroid_id, served_epoch, source_identity)`; future
payload extensions must use a new type or append-only compatible
contract rather than reordering this shape.

## The placement directory

The placement directory is a coordinator-local table that maps every
distributed row to its owning remote:

```sql
CREATE TABLE ec_spire_placement (
  index_oid       oid     NOT NULL,
  pk_value        bytea   NOT NULL,         -- canonical PK encoding
  node_id         integer NOT NULL,
  centroid_id     bigint  NOT NULL,
  served_epoch    bigint  NOT NULL,
  source_identity bytea   NOT NULL,         -- ADR-063 16-byte payload
  PRIMARY KEY (index_oid, pk_value)
);
CREATE INDEX ec_spire_placement_by_identity
  ON ec_spire_placement (index_oid, source_identity);
```

The placement directory is the authoritative `id → node_id` map for
coordinator-routed UPDATE/DELETE/PK-read. It is updated atomically with
remote INSERT/DELETE via two-phase commit.

It is **not** the same as the materialization catalog from
(superseded) ADR-065. The placement directory stores only the mapping
metadata required for write-routing; it does not store row data and is
not consulted on the read path. CustomScan reads route by centroid; the
placement directory only serves write routing and primary-key lookup.

## PK-keyed coordinator reads

Non-vector queries with a primary-key qual become coordinator-routed
reads:

```sql
SELECT cols FROM documents WHERE id = $1;
```

The coordinator looks up `node_id` in the placement directory, dispatches
the query to the owning remote, and returns the row. This is one
round-trip per PK lookup; for high-volume PK reads, application-side
caching is appropriate.

PK-keyed reads do not use CustomScan; they go through a coordinator-side
write/read forwarding mechanism that shares the placement-directory
lookup with INSERT/UPDATE/DELETE.

## Cross-shard non-PK reads

Non-vector queries without a PK qual, e.g.:

```sql
SELECT count(*) FROM documents WHERE published_at > '2024-01-01';
```

Require scatter-gather across all shards. This is **out of scope for
v1**. v1 supports vector-ordered reads (CustomScan), PK-keyed reads
(placement-directory lookup), and writes (coordinator-routed). Other
relational queries against the distributed logical relation return
either an empty result (from the coordinator's empty local heap) or a
clear error indicating scatter-gather is not supported.

A future ADR may add a Citus-style distributed query planner for
non-vector scatter-gather. v1 declares this out of scope.

## DDL on the distributed logical relation

DDL changes to the coordinator's logical relation (`ALTER TABLE`,
`CREATE INDEX` on a non-vector column, etc.) **must be applied to each
remote shard** by the operator. v1 does **not** automatically propagate
DDL.

The coordinator records the relation's column shape at relation-creation
time. Stage B endpoint identity / fingerprint already catches schema
drift between coordinator and remote at query time. An operator who
applies DDL to the coordinator but forgets to propagate to a remote
will see the fingerprint mismatch surface as a normal Stage B fault
(`endpoint_identity_mismatch`).

A future ADR may add coordinator-driven DDL propagation. v1 keeps DDL
operator-managed.

## Required invariants

- Coordinator-routed INSERT/UPDATE/DELETE on the distributed logical
  relation MUST be atomic: either the remote row change and the
  coordinator placement-directory update both commit, or neither commits.
- INSERTs MUST NOT silently fail when a remote is unreachable. Strict
  mode fails closed with `requires_remote_node_descriptor` or the
  appropriate Stage E fault category. Degraded mode (configured per
  session) MAY return a deferred-write status; the contract for
  deferred writes is part of the implementation packet.
- UPDATE of the embedding column MUST error with a clear message; it
  MUST NOT silently produce stale routing data by leaving the row on its
  old shard.
- The placement directory MUST be consulted on every coordinator-routed
  write path; it is the single source of truth for `id → node_id`.
- The bulk-load classification helper MUST return the same `node_id`
  that coordinator-routed INSERT would have chosen for the same
  embedding under the active epoch.

## What ships in v1

- Coordinator-routed INSERT with two-phase commit atomicity.
- Coordinator-routed UPDATE (non-embedding columns).
- Coordinator-routed DELETE.
- Coordinator-routed PK-keyed SELECT.
- Embedding-UPDATE rejection with clear error.
- Placement directory table and maintenance.
- `ec_spire_classify_centroid` helper.
- `ec_spire_register_placement_batch` primitive for bulk-load post-write
  registration.

## What is explicitly deferred to future ADRs

The following are out of v1 scope and will be addressed in separate ADRs
when their use cases warrant. They are listed so the deferral is visible.

- **Bulk-load tooling and CLI surface** — `ecaz` commands, parallel
  ingestion mechanism, error recovery for partial bulk-load failures.
  Separate task with its own packets.
- **Cross-shard UPDATE-as-move for embedding changes** (future ADR).
  Atomic cross-shard moves that update the embedding and migrate the
  row to its new home shard.
- **Cross-shard scatter-gather for non-vector queries** (future ADR).
  Citus-style distributed query planner integration for
  `SELECT ... WHERE non_indexed_column = ...` and aggregates over the
  distributed relation.
- **DDL propagation** (future ADR). Automatic propagation of
  `ALTER TABLE`, `CREATE INDEX`, `DROP INDEX` from the coordinator to
  all shards.
- **Foreign keys referencing the distributed relation** (future ADR).
- **Sequences and identity columns coordinated across shards** (future
  ADR). v1 requires application-side ID generation (UUID, application
  counter) or per-shard sequences with no global ordering guarantees.
- **Rebalance on centroid retraining** (future ADR). When the
  coordinator retrains routing centroids and some rows' assignments
  change, automatically migrate those rows to their new shards.
- **Multi-coordinator deployments** (future ADR). v1 assumes one
  coordinator.

## Consequences

- Applications can use the distributed `ec_spire` table much like a
  regular PostgreSQL table for the common write patterns (INSERT a row,
  UPDATE a non-embedding column, DELETE a row). No application-side
  shard awareness required.
- Bulk-load workflows have a documented escape hatch and a separate
  optimization track.
- Coordinator-side state (placement directory + routing centroids) is
  the single point of write coordination. Coordinator availability is a
  hard requirement for writes.
- Two-phase commit adds latency to every coordinator-routed write
  (typically one extra remote round-trip vs naive single-phase). For
  bulk workloads the application-routed path avoids this entirely.
- Embedding-UPDATE rejection is operationally simple but pushes the
  rare "I need to change my vector" case to a DELETE + INSERT
  application pattern. Acceptable for v1; a future ADR may add atomic
  cross-shard moves.

## Rejected Alternatives

### Application-routed writes as the v1 primary contract

Rejected. Operator preference is for transparent coordinator-routed
writes matching the user's mental model of a regular PostgreSQL table.
Application-routed writes are retained only as the bulk-load escape
hatch where coordinator routing is a throughput bottleneck.

### Single-phase commit for coordinator-routed writes

Rejected. Without two-phase commit, the placement directory can drift
from the remote heap when a coordinator crashes between the remote
commit and the local commit. The drift is silent and hard to detect.
Two-phase commit is the cost of atomic distributed writes; v1 pays it.

### Auto-DELETE + INSERT for embedding UPDATEs

Rejected for v1. An automatic cross-shard move requires its own
atomicity, visibility, and failure-recovery design that does not fit
v1. A clear error redirecting the application to explicit DELETE +
INSERT is operationally honest and unblocks v1 without precluding a
future ADR that lifts the restriction.

### Broadcast writes to all shards

Rejected. Storage cost scales as N (rows × shards) instead of staying
constant, defeating the storage-scale-out property ADR-067 was chosen to
deliver.
