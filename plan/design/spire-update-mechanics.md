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

The scheduler should treat those rows as advisory. The concrete scheduler is
not decided yet; viable first implementations are a manual
`ec_spire_maintain(index_oid)` SQL entrypoint, a VACUUM-time cleanup hook, or a
later background worker. Before publishing, whichever scheduler wins must
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

In the single-level foundation the parent routing object is the root, so the
routing rewrite cost scales with `nlists`, not with only the affected leaf. The
flat root object is still expected to be small enough for Phase 1, but split
rate decisions should treat this as a whole-root rewrite until recursive
hierarchy lands.

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
one parent first. Cross-parent merge is not a leaf-local merge: it is a
multi-parent coverage rewrite that allocates replacement PIDs and rewrites every
affected parent routing object plus the required higher-level routing objects.

## Rebalance

Rebalance keeps the logical partition identity only when coverage does not
change. For example, moving a leaf object to another local store or compacting
its row segments may reuse the PID and increment `object_version`.

For Phase 1, "coverage does not change" means the centroid stored in the parent
routing object remains byte-equal. A maintenance step that recomputes or drifts
that centroid changes routing semantics and is therefore a coverage rewrite:
split, merge, or split-of-one/merge-of-one style replacement with new PIDs.

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

Retained prior epochs remain queryable through their own placement directories.
After split or merge, old PIDs that disappeared from the new active routing
object must still be readable for scans pinned to the retired epoch until the
retention window and active-query rules make those old object tuples cleanup
candidates.

## Concurrency

The first implementation should use the same publish lock as insert and vacuum
cleanup. Concurrent scans keep using the previously active epoch until
root/control advances. Concurrent writers serialize at the publish boundary.

Replacement PIDs must come from the same root/control PID allocator cursor used
by insert. This allocator-cursor dependency is the main reason split, merge,
rebalance, insert, and vacuum cleanup share one publish lock in the first
implementation: the lock serializes both epoch publication and PID allocation.

Later batching or optimistic publish can relax this, but must preserve the same
epoch validation contract and deterministic `vec_id` dedupe semantics.
