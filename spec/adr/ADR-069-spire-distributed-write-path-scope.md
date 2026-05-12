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

The transparent v1 front door is intentionally narrow:

- Each distributed heap relation has at most one `ec_spire` index. Multiple
  `ec_spire` indexes on the same heap are rejected for coordinator-routed DML
  because routing centroids, placement rows, and tuple-payload dispatch are
  scoped to one index OID.
- The coordinator-visible primary key is one `bigint` column. Composite
  primary keys and non-`bigint` primary keys are deferred; they fail closed as
  `unsupported_pk_shape` rather than being encoded ambiguously.
- CTE-prefixed front-door statements are not supported in v1, including
  read-only `WITH` clauses. The planner hook rejects them as
  `unsupported_subquery_shape`; a later packet may distinguish read-only CTEs
  from modifying CTEs after the classifier can prove the rewritten target
  relation and predicate are still unambiguous.

The diagnostic helper `ec_spire_dml_frontdoor_classify_sql(sql text)` parses,
analyzes, and rewrites the supplied statement through PostgreSQL before it
invokes the front-door classifier. Standard PostgreSQL analysis errors, such
as missing relations, missing columns, or type mismatches, surface before any
SPIRE-specific diagnostic row is returned. This helper is an operator probe;
planner-hook execution must use catalog/relcache relation context and must not
call this SPI-backed diagnostic path.

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
- SPIRE prepared-transaction GIDs use
  `ec_spire_insert_<index_oid>_<node_id>_<served_epoch>_<top_xid>`.
  The `ec_spire_insert` prefix is historical and is shared by current
  coordinator-routed INSERT and DELETE prepares, so operators must not infer
  operation type from the prefix. The GID deliberately omits backend pid
  because a crashed backend pid is not a stable recovery key; `top_xid` is
  allocated with `GetTopTransactionId()` when the remote prepare starts so the
  local transaction has a durable identity for correlation with logs and other
  coordinator-side evidence.

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

The first implementation surface is
`ec_spire_forward_coordinator_update_tuple_payload(index_oid, pk_column,
pk_value, row_payload, updated_columns)`. It looks up `node_id` and
`served_epoch` in `ec_spire_placement`, reuses the remote descriptor and
conninfo-secret dispatch gate, and calls the remote
`ec_spire_remote_update_tuple_payload(...)` endpoint. The remote endpoint
updates only the explicit non-PK columns supplied in `updated_columns`,
matching the row by the v1 canonical bigint primary-key bytes. This is the
forwarding primitive. If the placement row points at local node `0`, the
same helper applies the payload update directly to the coordinator heap
instead of attempting remote dispatch. Transparent
`UPDATE documents SET ... WHERE id = ...` is implemented by a planner-hook
plan-tree replacement: the supported statement is planned as a top-level
`EcSpireDistributedScan` CustomScan that invokes this primitive directly,
increments PostgreSQL's processed-row count from the primitive result, and
returns no tuple.

Non-embedding UPDATE has at-most-once visible semantics from the
application's perspective: if the connection drops after the remote
commits but before the coordinator receives the result, retrying may
apply the statement again. Idempotent assignments such as
`SET title = $new_value WHERE id = $id` are safe to retry; non-idempotent
statements such as counter increments need application-level
idempotency.

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

V1 DELETE collision policy is idempotent not-found success. If the
coordinator placement row is already absent, the DELETE returns zero affected
rows and does not dispatch remotely. If the placement row exists but the owning
heap row is already absent, the DELETE still removes the placement row and
returns zero affected rows. This matches PostgreSQL DELETE semantics for a
predicate that no longer matches a visible row while ensuring stale placement
directory entries do not survive a cleanup attempt.

The implementation surface is
`ec_spire_prepare_coordinator_delete_tuple_payload(index_oid, pk_column,
pk_value)`. It looks up `node_id` and `served_epoch` from
`ec_spire_placement`, prepares a remote
`ec_spire_remote_delete_tuple_payload(...)` call in a remote transaction,
then deletes the coordinator placement row in the local transaction. The
existing transaction callback resolves the remote prepared transaction on
local commit or abort, so the remote heap delete and placement-directory
delete commit or roll back together. If the placement row points at local
node `0`, the same helper deletes directly from the coordinator heap and
removes the placement row without opening a remote transaction.

Transparent `DELETE FROM documents WHERE id = ...` uses the same
planner-hook plan-tree replacement shape as transparent UPDATE: the
supported statement is planned as a top-level `EcSpireDistributedScan`
CustomScan that invokes the delete primitive directly, increments
PostgreSQL's processed-row count from the primitive result, and returns no
tuple.

### Transparent DML front-door limitations

The v1 transparent UPDATE/DELETE front door intentionally bypasses
PostgreSQL `ModifyTable` for supported single-row, bigint-PK statements.
That is the integration point that lets SPIRE route remote-owned rows that
are not present in the coordinator heap. The tradeoff is that v1 supports
only plain rowcount semantics for the rewritten statements:

- `RETURNING` is not supported on transparent distributed UPDATE/DELETE.
- Coordinator table row-level triggers do not fire for transparent
  distributed UPDATE/DELETE.
- Statement-level transition tables are not populated for transparent
  distributed UPDATE/DELETE.

These constraints are part of the v1 contract rather than incidental
implementation gaps. A future ADR may introduce a richer ModifyTable-like
integration if SPIRE needs trigger or transition-table semantics for
distributed rows.

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

Packet `30841` wires this rejection into
`ec_spire_forward_coordinator_update_tuple_payload(...)` by detecting updates
to the `ec_spire` index key column before placement lookup or remote dispatch.
The transparent `UPDATE ... WHERE pk = ...` front door remains a follow-up, but
the shared coordinator UPDATE primitive now fails with the documented error and
hint.

A future ADR may add atomic cross-shard UPDATE-as-move. v1 keeps
the contract narrow.

### Bulk-load escape hatch (separate task)

Coordinator-routed INSERT is the correctness-first default for ordinary
application writes. It pays one remote transaction, one remote
`PREPARE TRANSACTION`, one coordinator placement-directory write, and one
remote prepared-transaction resolution per affected remote row. That cost buys
atomic visibility between the remote heap row and the coordinator placement
directory, but it is not the highest-throughput ingestion path for workloads
that can tolerate a bounded post-write placement-registration window.

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
contracts. Batch placement registration is transactional within the calling
session: entries from one call become visible together at commit and are not
visible after rollback. Partial visibility is therefore an operational boundary
between committed bulk-load batches or tool runs, not within one registration
transaction. While the batch placement registration has not committed, those
directly loaded rows are outside coordinator-routed SPIRE read eligibility; the
bulk-load operator must either keep readers away from the partially registered
dataset or accept that searches may omit rows whose placement entries are not
yet visible.

The same classification and placement-entry preparation used by
coordinator-routed INSERT is exposed as
`ec_spire_plan_coordinator_insert(index_oid, pk_value, embedding,
source_identity)`. It is a side-effect-free planning primitive: it
validates the canonical primary-key bytes and source identity, classifies
the embedding, and returns the placement tuple fields. The later
coordinator-routed INSERT executor must persist that placement tuple only
after remote prepare succeeds.

The remote dispatch readiness gate for the later mutating executor is
exposed as `ec_spire_plan_coordinator_insert_dispatch(index_oid, node_id,
served_epoch)`. It reuses the Stage C remote-node descriptor and external
conninfo-secret contract, checks the descriptor's served-epoch window, and
returns the libpq dispatch action for the 2PC protocol. It does not expose
raw conninfo, open a socket, forward a row, write `ec_spire_placement`, or
prepare a remote transaction.

Every remote PostgreSQL instance that accepts coordinator-routed SPIRE writes
must run with `max_prepared_transactions` greater than zero and enough free
prepared-transaction slots for peak concurrent SPIRE remote prepares plus any
non-SPIRE prepared transactions on the same instance. PostgreSQL requires a
restart after changing this setting. If remote `PREPARE TRANSACTION` fails
because prepared transactions are disabled or capacity is exhausted, the
coordinator wraps the remote error with a SPIRE-specific hint naming
`max_prepared_transactions` as the readiness requirement. Remote descriptor
registration performs a nonblocking preflight when the descriptor's
`conninfo_secret_name` resolves: it emits a NOTICE-level operator message if
the remote is unreachable, if the setting cannot be read, or if the remote
reports `max_prepared_transactions = 0`. Registration remains nonblocking so
secret rollout and descriptor rollout can remain decoupled, but the message is
a coordinator-routed write-readiness blocker for operators.

The mutating executor's first internal 2PC primitive sends the remote
INSERT inside a remote transaction, issues `PREPARE TRANSACTION`, and
registers transaction callbacks so the prepared remote transaction is
resolved according to the coordinator transaction outcome. The local
placement row is inserted only after the remote prepare succeeds. If the
coordinator transaction aborts before commit, the remote prepared
transaction is rolled back; if the coordinator commits, the remote
prepared transaction is committed. A failed callback resolution leaves the
prepared transaction visible to normal PostgreSQL prepared-transaction
operator recovery. Operators identify SPIRE prepared transactions on the
remote with `pg_prepared_xacts.gid LIKE 'ec_spire_insert_%'`, parse the
stable `(index_oid, node_id, served_epoch, top_xid)` identity from the GID,
and resolve only after matching the coordinator transaction outcome to the
placement-directory state for the affected primary key. For INSERT recovery,
a committed coordinator transaction with the expected placement row means
`COMMIT PREPARED`; an aborted or absent placement outcome means
`ROLLBACK PREPARED`. For DELETE recovery, a committed coordinator
transaction with the placement row removed means `COMMIT PREPARED`; if the
placement row remains because the coordinator transaction did not commit,
use `ROLLBACK PREPARED`. If the coordinator transaction outcome or affected
primary key cannot be established, leave the prepared transaction in place
for manual escalation instead of guessing.

The remote shard exposes `ec_spire_remote_insert_tuple_payload(index_oid,
row_payload, requested_columns)` as the typed INSERT endpoint the
coordinator can call inside that prepared remote transaction. The endpoint
derives the remote heap relation from the remote SPIRE index, validates the
requested column list against ordinary heap attributes, projects the JSON
payload through PostgreSQL type input, and inserts exactly the named
columns. The coordinator still owns classification, placement-directory
staging, and transaction resolution.

The coordinator-side remote-prepare primitive builds its mutating remote
statement as a call to that endpoint, using the remote index regclass from
the active remote-node descriptor and the executor-provided JSON tuple
payload plus explicit column list. This keeps table-specific INSERT
construction on the coordinator side out of the wire contract.

`ec_spire_prepare_coordinator_insert_tuple_payload(index_oid, pk_value,
embedding, source_identity, row_payload, requested_columns)` composes the
v1 INSERT operation before the transparent DML hook is installed. It
classifies the embedding, prepares the typed remote tuple-payload INSERT,
and stages the placement-directory row in the coordinator transaction.
After staging succeeds it reports
`remote_insert_prepared_pending_local_commit` with next step
`await_local_commit`; the remote prepared transaction is not durable or
visible until the caller's coordinator transaction commits and resolves the
remote prepared transaction. Before staging the placement row, the helper
captures the remote index active epoch and endpoint fingerprint inside the
remote transaction and advances the coordinator remote-node descriptor in
the same local transaction. If local staging aborts, the descriptor refresh
rolls back with the placement row and the existing transaction callback
rolls back the remote prepared transaction.
The eventual INSERT hook must call the same operation after constructing
the canonical primary-key bytes, ADR-063 source identity, JSON tuple
payload, and explicit column list from the executor tuple.
The packet `30836` PG18 multicluster smoke validates the helper path by
committing a remote row, staging its coordinator placement, proving the
descriptor epoch/identity refresh happens automatically, and reading the
row back through `EcSpireDistributedScan`.

The first transparent INSERT front door is trigger-based. Operators call
`ec_spire_enable_coordinator_insert(table_oid, index_oid, pk_column,
embedding_column, source_identity_column)` to install a `BEFORE INSERT`
row trigger. The trigger supports the v1 narrow shape: bigint primary key
columns encoded with PostgreSQL's `int8send`, an `ecvector` embedding
column cast to `real[]`, and a 16-byte `bytea` source-identity column. It
forwards the row through
`ec_spire_prepare_coordinator_insert_tuple_payload(...)` and returns
`NULL`, so remote-owned rows are not mirrored in the coordinator heap.
The trigger uses the same helper path, including automatic remote
descriptor epoch/identity refresh.

UUID primary keys and non-`bytea` trigger source-identity columns are
deferred. ADR-063 still allows `uuid` source identity for index INCLUDE
columns, but the v1 coordinator-routed INSERT trigger pins its front-door
wire shape to a canonical bigint primary key plus exact 16-byte `bytea`
source identity. A future packet can add `uuid_send` primary-key encoding
and a richer trigger/hook contract without changing the existing bigint
shape.

Descriptor refresh uses the remote endpoint's post-INSERT descriptor
generation as a monotonic guard. If concurrent coordinator INSERTs for the
same `(index_oid, node_id)` race and a newer descriptor generation wins
first, the older transaction can fail to advance the descriptor and roll
back. That race raises SQLSTATE `40001` (`serialization_failure`) with the
message
`ec_spire_register_remote_node_descriptor descriptor_generation must advance
existing descriptor_generation`; callers should retry the whole coordinator
write after the winning descriptor refresh commits. The failed transaction has
not published its placement row, and the existing transaction callback rolls
back its remote prepared transaction.
Stage F can replace this with an explicit live-row compatibility check or
per-node INSERT serialization if concurrent INSERT throughput requires it.

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

For v1, `centroid_id` is the active-epoch routing leaf pid selected by
`ec_spire_classify_centroid`. Consumers should treat it as an opaque
routing-leaf identity scoped to `(index_oid, served_epoch)`, not as a
stable semantic centroid across retraining or rebalance. Future epoch
retraining must either rewrite affected placement rows or reclassify
them before using this value for pruning.

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
They are read-only and idempotent: if a connection drops mid-read, callers
can retry safely. The primitive rejects multi-row matches as schema drift,
because v1 PK lookup expects at most one row for the canonical primary-key
bytes.

Packet `30840` adds the first PK-read primitive:
`ec_spire_forward_coordinator_select_tuple_payload(index_oid, pk_column,
pk_value, requested_columns)`. It looks up placement by canonical bigint
primary-key bytes, serves `node_id = 0` placements from the coordinator heap,
and forwards remote placements to
`ec_spire_remote_select_tuple_payload(...)` through the same descriptor,
conninfo-secret, epoch-window, timeout, and advisory-governance dispatch path
used by UPDATE. The primitive returns a JSON tuple payload for the requested
columns; transparent `SELECT ... WHERE pk = ...` integration remains a
planner/view-hook follow-up.
Packet `30846` adds the defensive `selected_count > 1` guard for both local
and remote branches.

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
