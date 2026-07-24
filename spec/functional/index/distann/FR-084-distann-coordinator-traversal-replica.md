---
id: FR-084
title: Distann Coordinator Traversal Replica
type: FR
object: entity
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-083"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-084: Distann Coordinator Traversal Replica

## Description

An authoritative ec_distann coordinator MAY hold one optional, rebuildable
traversal replica for its active Published epoch. The replica is a derived
performance object, never an epoch, owner generation, row-tier authority, or
payload store.

The replica SHALL contain exactly the graph record and full-precision source
vector for every vec_id in the active generation. Final projected rows SHALL
still be materialized from the hash owner under FR-078 and FR-081.

The existing owner traversal SHALL remain the default and correctness fallback.

## Properties

| Property | Type | Rule |
| --- | --- | --- |
| logical identity | `(index_oid, logical_index_uuid, build_id)` | Primary key; one replica candidate per physical generation. |
| epoch identity | `epoch_fingerprint byte[34]` | Canonical FR-082 v2 fingerprint and unique per logical index. |
| descriptor identity | `generation_descriptor_digest byte[32]` | Must equal the active build candidate and every owner generation. |
| content identity | `content_digest byte[32]` | Present only after the complete deterministic stream validates. |
| state | `Building \| Ready \| Stale \| Retiring` | Only transitions in the lifecycle table are valid. |
| physical storage | permanent heap plus unique B-tree directory | WAL-logged and coordinator-local; OIDs never enter the epoch manifest. |
| owner coverage | one row per roster ordinal | Every owner records its copied count and owner content digest before Ready. |
| payload content | none | The replica relation has no projected/source-row payload columns. |

The version-1 replica heap SHALL contain:

| Column | PostgreSQL type | Rule |
| --- | --- | --- |
| `vec_id` | `bigint` | Bit-preserving u64 identity; unique through the directory. |
| `owner_ordinal` | `integer` | FR-078 hash owner and within the immutable roster. |
| `graph_record` | `bytea` | Exact FR-076 physical graph record from that owner. |
| `exact_vector` | `bytea` | Exactly `dimensions * 4` bytes of finite IEEE-754 f32 words in little-endian order. |

Rows SHALL be inserted and digested in ascending
`(owner_ordinal, stored bigint vec_id)` order. `vec_id` retains its u64 bit
pattern, but the physical B-tree's signed `bigint` order is canonical so
bounded keyset pagination and digest replay use the same order. The version-1
content digest is:

```text
SHA-256(
  "ec_distann_traversal_replica_v1\0" ||
  version:u16 ||
  logical_index_uuid:byte[16] ||
  build_id:byte[16] ||
  epoch_fingerprint:byte[34] ||
  generation_descriptor_digest:byte[32] ||
  dimensions:u16 ||
  graph_degree:u16 ||
  neighbor_codec_kind:u8 ||
  owner_count:u32 ||
  expected_record_count:u64 ||
  for each row:
    owner_ordinal:u32 ||
    vec_id:u64 ||
    graph_record_length:u32 || graph_record ||
    exact_vector_length:u32 || exact_vector
)
```

All integers are little-endian. Lengths SHALL be checked before allocation or
hashing. A repeated or out-of-order owner/vec_id, non-owner vec_id, malformed
graph record, mismatched graph-record vec_id, non-finite vector, wrong vector
length, incomplete owner, or cardinality mismatch SHALL prevent Ready.

## Lifecycle

| Current | Operation | Next | Required behavior |
| --- | --- | --- | --- |
| absent | build active epoch | Building | Create hidden relation and directory; record immutable identity before copying. |
| Building | verified complete stream | Ready | Commit owner coverage, content digest, byte counts, then expose eligibility atomically. |
| Building | build failure/restart | absent or Building | Cleanup is idempotent; a replay resumes only the identical identity or rebuilds from zero. |
| Ready | first published-generation mutation | Stale | Commit on a dedicated coordinator control connection before returning the stable retryable error; dispatch no owner mutation on this attempt. |
| Ready or Stale | active epoch changes or explicit retire | Retiring | New scans cannot select the replica. |
| Retiring | replica scan pins drain | absent | Drop derived relations and delete catalog rows idempotently. |

A scan SHALL first pin and revalidate the FR-082 active epoch, then select a
Ready catalog row matching all of `(index_oid, logical_index_uuid, build_id,
epoch_fingerprint, generation_descriptor_digest)`. Replica selection SHALL add
no owner request.

A scan that selected Ready before invalidation MAY finish against its pinned
immutable replica. New scans observe Stale. Replica pins participate in the
same fingerprint retirement fence as owner traversal scans.

## Traversal and Fallback

Replica traversal SHALL use the same persisted head, ordered seeds, BW/H,
RaBitQ neighbor scoring, exact-distance calculation, tombstone behavior,
convergence rule, work cap, and final ordering as owner traversal. A remote
owner heap TID is never interpreted locally.

Missing, Building, Stale, Retiring, incomplete, corrupt, or identity-mismatched
replicas SHALL select owner traversal before local work begins.

If a replica read fails after traversal begins, the coordinator SHALL discard
all replica frontier, visited, hit, and attribution state and restart once from
the beginning through owner traversal under the same pinned epoch. No partial
replica prefix may be reused or returned. Owner-path failure semantics remain
those of FR-079 through FR-082.

## Mutation Invalidation

The Ready-to-Stale transition SHALL be committed independently of the caller's
failing DML transaction. Version 1 uses a dedicated loopback coordinator
control connection in autocommit mode. The caller SHALL:

1. request conditional `Ready -> Stale` for the exact active identity;
2. wait for that control transaction to commit;
3. dispatch no owner write; and
4. return `EC_REPLICA_INVALIDATED`, a stable retryable error.

The retry observes Stale and runs the ordinary owner-authoritative mutation.
If invalidation cannot commit, the mutation fails closed with no owner write.
A crash after the control commit is safe. The catalog update SHALL be the only
lock acquired by the side transaction, preventing a lock dependency on the
outer DML transaction.

## Operator Surface

Task 198 SHALL provide:

```text
ec_distann_build_traversal_replica(index_regclass regclass) returns bytea
ec_distann_traversal_replica_status(index_regclass regclass)
  returns table (...identity, state, counts, digests, relation bytes...)
ec_distann_retire_traversal_replica(index_regclass regclass) returns void
ec_distann_reclaim_traversal_replica(index_regclass regclass) returns boolean
```

Build, retire, and reclaim require the index owner or superuser, use a fixed
SECURITY DEFINER search path, revoke PUBLIC execute, and reject non-authoritative
or multi-coordinator topology.

## Acceptance Criteria

- **FR-084-AC-1:** Ready requires exact global cardinality, complete roster
  coverage, unique/hash-correct vec_ids, descriptor equality, and reproducible
  content digest.
- **FR-084-AC-2:** Same-query replica and owner traversals return identical
  ordered hits and counters before owner payload materialization.
- **FR-084-AC-3:** Wrong fingerprint/digest/state never selects the replica.
- **FR-084-AC-4:** A mid-replica fault restarts wholly on the owner path.
- **FR-084-AC-5:** Exactly one first mutation attempt returns
  `EC_REPLICA_INVALIDATED`; its retry reaches the owner mutation path.
- **FR-084-AC-6:** Build/retire/reclaim and crash replay are idempotent and do
  not reclaim while a fingerprint scan pin exists.
- **FR-084-AC-7:** Replica storage, build/WAL/copy cost, cache residency,
  recall, latency, and traversal attribution are measured by the Task 198
  checked-in suite before any promotion.
