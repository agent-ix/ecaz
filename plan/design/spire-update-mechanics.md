# SPIRE Update Mechanics Plan

Status: Phase 2 planning checkpoint for Task 30
Date: 2026-05-03
Scope: local single-store split/merge/rebalance mechanics over SPIRE
partition objects

This note translates the LIRE/SPFresh-style online update mechanics into the
PostgreSQL relation-backed storage model chosen by ADR-049 and the Phase 0
partition-object design. It does not implement a scheduler or background
worker; it defines the publication shape future code must follow.

## Invariants

- Published partition objects are immutable. Maintenance writes replacement
  objects and publishes a new epoch.
- PIDs identify logical partitions inside one index. Object versions identify
  replacement physical objects for the same logical PID.
- Heap TIDs are row locators only. `vec_id` remains the dedupe identity across
  delete deltas, replacement leaves, future boundary replicas, and remote
  candidate merge.
- Local single-store Phase 1 is strict by default. A split, merge, or rebalance
  publish must fail closed if any required placement is unavailable.
- Publication order stays the existing order: write objects, write placement
  entries, write manifest bundle, validate, then advance root/control.

## Maintenance Inputs

The read-only trigger surface is `ec_spire_index_leaf_snapshot(index_oid)`.
The first local rules are:

- split candidate: effective assignments are at least
  `max(32, 4 * ceil(total_effective_assignments / active_leaf_count))`
- merge candidate: effective assignments are at or below
  `floor(ceil(total_effective_assignments / active_leaf_count) / 4)`

The scheduler should treat those rows as advisory. Before publishing it must
reload the active epoch under the update/vacuum publish lock and re-check that
the selected leaf PIDs still satisfy the expected state.

## Split

A split changes partition coverage, so it allocates new child leaf PIDs rather
than reusing the old PID with a different semantic meaning.

1. Load the active epoch snapshot and visible rows for the split leaf.
2. Train two or more child centroids from those visible source vectors.
3. Allocate replacement leaf PIDs and write new V2 leaf objects containing the
   visible assignments routed to those child centroids.
4. Rewrite the parent routing object so the old child PID is replaced by the
   new child PIDs and their centroids.
5. Publish a replacement epoch whose active placement directory includes the
   unchanged objects, the rewritten routing object, and the new leaf objects.
6. Retire the previous active manifest. Old leaf object tuples become cleanup
   candidates after retention.

The old split leaf PID is not reused for a new coverage region. It remains
referenced only by retained prior epochs.

## Merge

A merge also changes coverage, so it allocates a replacement leaf PID for the
merged region rather than silently broadening one survivor PID.

1. Select sibling or nearby sparse leaves from the same routing parent.
2. Load visible rows from the selected leaves and any active deltas attached to
   them.
3. Write one replacement V2 leaf object for the merged assignment set.
4. Rewrite the parent routing object by removing the merged child PIDs and
   adding the replacement PID with a recomputed centroid.
5. Publish the replacement epoch and retire the previous active manifest.

If future recursion introduces internal routing levels, merge is applied within
one parent first. Cross-parent merge requires a higher-level routing rewrite and
must be treated as a rebalance, not a leaf-local merge.

## Rebalance

Rebalance keeps the logical partition identity when coverage does not change.
For example, moving a leaf object to another local store or compacting its row
segments may reuse the PID and increment `object_version`.

Rebalance must not change the centroid boundary represented by that PID. If it
does, it is a split or merge and must allocate replacement PIDs.

## Deltas and Visibility

Before split or merge publication, maintenance must fold active insert/delete
deltas into the replacement V2 leaf objects using the same visible-assignment
logic as vacuum compaction:

- insert-delta rows that survive delete suppression become primary leaf rows
  with the delta-insert flag cleared
- delete-delta rows suppress matching `vec_id`s and are not written to base
  leaves
- the new active placement directory contains no delta objects for the affected
  old leaves

Unchanged leaves and routing objects may be carried forward by reference with
their placement epoch restamped, matching current insert/vacuum replacement
epochs.

## Concurrency

The first implementation should use the same publish lock as insert and vacuum
cleanup. Concurrent scans keep using the previously active epoch until
root/control advances. Concurrent writers serialize at the publish boundary.

Later batching or optimistic publish can relax this, but must preserve the same
epoch validation contract and deterministic `vec_id` dedupe semantics.
