---
id: FR-059
title: SPIRE Coordinator Routed DML and 2PC
type: functional-requirement
artifact_type: FR
status: APPROVED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/FR-055"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-057"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-059: SPIRE Coordinator Routed DML and 2PC

## Requirement

Distributed SPIRE SHALL support coordinator-routed INSERT, non-embedding
UPDATE, DELETE, and PK-keyed SELECT for the v1 narrow table shape while keeping
remote heap changes and coordinator placement-directory state atomic where both
sides mutate.

## Supported V1 Front Door

| Operation | Contract |
| --- | --- |
| INSERT | Trigger/helper classifies embedding, prepares remote insert, stages placement row, resolves remote prepared xact on local outcome. |
| UPDATE | Non-embedding columns are forwarded to the owning node from placement-directory lookup. |
| DELETE | Owning node delete and placement-directory delete are coordinated with remote prepared transaction. |
| PK SELECT | Placement-directory lookup dispatches one remote/local tuple-payload read. |
| Embedding UPDATE | Rejected with clear error and hint to use DELETE + INSERT. |

V1 SHALL require one `bigint` primary key for transparent DML helpers. Composite
PKs, float/numeric PKs outside `int8send` canonical encoding, CTE-prefixed
front-door statements, `RETURNING`, coordinator row triggers, transition
tables, cross-shard non-PK reads, DDL propagation, and embedding moves are
deferred.

## INSERT 2PC Flow

```mermaid
sequenceDiagram
    participant App as Application
    participant Coord as Coordinator
    participant Remote as Remote node
    participant Intent as Prepared-xact intent
    participant Place as ec_spire_placement

    App->>Coord: INSERT logical row
    Coord->>Coord: classify centroid and source identity
    Coord->>Intent: record prepare_requested
    Coord->>Remote: INSERT row; PREPARE TRANSACTION gid
    Remote-->>Coord: prepare acknowledged
    Coord->>Intent: mark prepare_acked
    Coord->>Place: stage placement row
    Coord->>Intent: pre-commit mark commit_local
    Coord->>Remote: COMMIT PREPARED on local commit
```

## Prepared Transaction Contract

SPIRE prepared transaction GIDs SHALL use:

```text
ec_spire_<operation>_<index_oid>_<node_id>_<served_epoch>_<top_xid>_<branch_seq>
```

`operation` SHALL be `insert`, `update`, or `delete`. `branch_seq` SHALL be a
positive coordinator-local sequence number allocated per remote mutation branch
inside the top transaction. The tuple
`(index_oid, node_id, served_epoch, top_xid, branch_seq)` SHALL be unique for
every prepared remote transaction, including repeated row mutations against the
same index, node, and epoch in one coordinator transaction.

Legacy single-branch GIDs with the historical `ec_spire_insert_...` prefix MAY
be recognized by recovery tooling only for migration or diagnostic
compatibility. New prepares SHALL use the operation-bearing form above.

`ec_spire_remote_prepared_xact_intent` SHALL record:

| Column | Rule |
| --- | --- |
| `index_oid` | coordinator SPIRE index |
| `node_id` | remote node |
| `served_epoch` | epoch used for the remote prepare |
| `xid` | coordinator top transaction ID |
| `operation` | `insert`, `update`, or `delete` |
| `branch_seq` | positive branch sequence unique within the coordinator top transaction |
| `gid` | SPIRE GID |
| `intent_state` | `prepare_requested`, `prepare_acked`, `commit_local`, or `rollback_local` |

The operator-driven reaper SHALL roll back orphaned SPIRE prepared
transactions only when the coordinator top transaction is no longer live and
the intent state is not `commit_local`. Entries marked `commit_local` require
manual outcome confirmation if remote commit resolution failed.

## Acceptance Criteria

### FR-059-AC-1

Coordinator-routed INSERT and DELETE commit or roll back remote heap state and
coordinator placement-directory state together.

### FR-059-AC-2

The v1 front-door limitations are explicit and fail closed rather than
silently executing unsupported distributed semantics.

### FR-059-AC-3

The prepared-xact GID, intent states, lost-ack recovery window, and reaper
decision rule are defined well enough for operator recovery and repeated
branches cannot collide in one coordinator transaction.

### FR-059-AC-4

Coordinator-routed INSERT defines source identity, target node selection,
remote payload construction, placement-directory registration, and 2PC ordering.

### FR-059-AC-5

PK-keyed UPDATE and DELETE route through the placement directory and validate
that exactly the v1 supported table shape is being modified.

### FR-059-AC-6

Embedding-changing UPDATE is rejected with an explicit distributed SPIRE error
instead of silently moving rows across shards.

### FR-059-AC-7

PK-keyed SELECT can use placement-directory routing without pretending to be a
general cross-shard SQL planner.

### FR-059-AC-8

Bulk-load registration is identified as an operational mode outside the
transparent coordinator DML front door.

### FR-059-AC-9

Every unsupported DML shape fails before remote mutation and points operators at
the v1 SPIRE distributed DML contract.
