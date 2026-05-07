---
id: FR-043
title: SPIRE Update, Split, Merge, and Cleanup Lifecycle
type: functional-requirement
artifact_type: FR
status: DRAFT
object_type: process
relationships:
  - target: "ix://agent-ix/tqvector/US-017"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/US-020"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/FR-038"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/FR-041"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-043: SPIRE Update, Split, Merge, and Cleanup Lifecycle

## Requirement

`ec_spire` SHALL define how inserts, deletes, updates, partition split/merge, rebalancing, vacuum, and epoch cleanup modify partition objects without making active queries observe incoherent index state.

## Behavior

1. The first baseline MAY prioritize the easiest path that proves functionality, including offline build plus simple insert/delete support.
2. Phase 1 SHALL treat published partition objects as immutable and represent local inserts/deletes through epoch-published delta objects or replacement object versions.
3. Inserts SHALL assign new vectors to one or more leaf PIDs according to the current router and boundary-replication policy.
4. Deletes SHALL remove or tombstone assignment rows without breaking active epoch reads.
5. Updates SHALL be represented as delete-old plus insert-new unless a narrower optimization is accepted later.
6. Split and merge operations SHALL create replacement partition objects and publish hierarchy/placement changes through an epoch transition.
7. Vacuum SHALL compact tombstones and reclaim obsolete partition-object versions only after retention and active-query checks pass.
8. Rebalance SHALL copy or rewrite partition objects to target stores or nodes, then publish a placement epoch.
9. Update and vacuum paths SHALL keep stored heap TIDs aligned with live tuple locators, including HOT/UPDATE movement, or mark affected assignment rows stale until repair.
10. Failed split, merge, rebalance, or compaction jobs SHALL leave the active epoch unchanged and expose failed-job state for retry or cleanup.

## Delta Schema

```text
spire_delta_row
  index_oid oid
  target_epoch bigint
  pid bigint
  op insert | delete | update | boundary_replica
  vec_id bytea
  heap_tid tid
  encoded_payload bytea
  flags int

spire_rewrite_job
  index_oid oid
  job_id bigint
  kind split | merge | rebalance | compact
  source_pids bigint[]
  target_pids bigint[]
  state pending | running | ready_to_publish | published | failed
```

## Update Lifecycle

```mermaid
flowchart TD
    Insert["insert/update/delete arrives"]
    Delta["write live delta or replacement object"]
    Trigger["check split/merge/rebalance triggers"]
    Build["build replacement partition objects"]
    Publish["publish new epoch/manifest"]
    Cleanup["cleanup old versions after retention"]

    Insert --> Delta
    Delta --> Trigger
    Trigger -->|"not needed"| Publish
    Trigger -->|"needed"| Build --> Publish
    Publish --> Cleanup
```

## Split/Merge Sequence

```mermaid
sequenceDiagram
    participant Upd as Update worker
    participant Old as Old partition objects
    participant New as New partition objects
    participant Root as Hierarchy metadata
    participant Epoch as Epoch publisher

    Upd->>Old: read candidates for split/merge
    Upd->>New: write replacement objects
    Upd->>Root: prepare parent/child metadata changes
    Upd->>Epoch: publish replacement epoch
    Epoch-->>Upd: old objects retained until safe
```

## Acceptance Criteria

### FR-043-AC-1

Inserts and deletes against an active strict-mode epoch either become visible to subsequent searches through a published epoch-safe path or fail explicitly.

### FR-043-AC-2

Split and merge never silently change PID child/leaf meaning for active strict-epoch queries.

### FR-043-AC-3

Vacuum and cleanup can prove an old epoch or partition-object version is no longer needed before reclaiming it.

### FR-043-AC-4

A failed split, merge, rebalance, or compaction job does not change the active epoch and remains diagnosable for retry or cleanup.

### FR-043-AC-5

Update and vacuum processing handles heap-TID invalidation by repairing the assignment row locator, tombstoning the stale assignment, or suppressing the candidate with diagnostics.
