# SPIRE Recursive Hierarchy Design

Status: Phase 3 design checkpoint for Task 30
Date: 2026-05-05
Scope: local single-store recursive routing over existing SPIRE partition
objects

This note defines the conservative recursion contract for SPIRE before the
implementation extends the landed single-level foundation. It is subordinate to
ADR-049, the Phase 0 partition-object storage design, and the Phase 2 update
mechanics plan.

## Goals

Phase 3 adds recursive IVF-on-centroids without changing the Phase 1/2
publication contract:

- Published partition objects stay immutable.
- A published epoch manifest remains the compatibility boundary for root,
  internal, leaf, delta, and placement entries.
- The first implementation remains local single-store and strict by default.
- Existing single-level `ec_spire` build, scan, insert, vacuum, and manual
  maintenance behavior must remain valid.

The first recursive implementation should prove a small hierarchy shape before
adding boundary replication, top-level graph routing, multi-store placement, or
product-scale measurements.

## Level Numbering

Level numbers describe distance from data-bearing leaves:

```text
level 0: leaf partition objects
level 1: routing objects whose children are leaves
level N: routing objects whose children are level N - 1 routing objects
```

The root is the unique routing object with `kind = Root`, `parent_pid = 0`, and
the maximum active hierarchy level for the epoch. A single-level IVF index is
therefore already a valid depth-1 hierarchy: root at level 1, leaves at level 0.

Internal routing objects use `kind = Internal`, `level > 0`, and nonzero
`parent_pid`. Leaf and delta objects use `level = 0`. Delta objects attach to a
leaf PID through `parent_pid` and never participate as routing children.

This is a semantic tightening of the current persisted shape. The existing
single-level root object already stores `level = 1`; existing leaves are treated
as level 0 for recursion purposes even where older helper names still call them
"single-level" objects.

## PID Invariants

PIDs remain index-local unsigned 64-bit identifiers allocated from
root/control metadata. `0` is invalid except as the root object's
`parent_pid`.

For every active epoch:

- Exactly one available root routing object exists.
- Every non-root active object has a nonzero `parent_pid`.
- A routing object's children are PIDs, not tuple offsets, heap rows, or
  PostgreSQL declarative partitions.
- Every child PID referenced by an active routing object must have an active
  manifest entry and an available placement in strict mode.
- A child referenced by a routing object at level `L` must be either:
  - a leaf object at level 0 when `L = 1`
  - an internal routing object at level `L - 1` when `L > 1`
- Routing children are unique within one parent object.
- Leaf PIDs are not silently reused for changed coverage. Split and merge
  continue to allocate replacement leaf PIDs when coverage changes.

The recursive build coordinator may allocate fresh internal PIDs while building
one unpublished epoch draft. Those PIDs become visible only when the epoch
manifest and root/control publish succeed.

## Routing Object References

The existing flat routing object body remains the first recursive reference
format:

```text
routing object
  header.kind            Root | Internal
  header.pid             parent routing PID
  header.level           child level + 1
  header.parent_pid      0 for root, otherwise containing parent PID
  dimensions             centroid dimension
  child_count
  repeated child entry:
    centroid_ordinal
    child_pid
    centroid[dimensions]
```

The centroid stored beside each child describes that child's coverage at the
parent's routing level. For `level = 1`, child centroids describe leaf vector
clusters. For `level > 1`, child centroids describe clusters of lower-level
centroids materialized during recursive build.

The first recursive scan implementation should verify child object kind and
level through the active snapshot before descending. A malformed strict epoch
must fail closed rather than route through a shape-mismatched child.

## Per-Level Parameters

Recursive-capable metadata must preserve the parameters used at each routing
level. The first durable shape should be diagnostic-friendly even if the bytes
are initially embedded in a root/hierarchy metadata object or manifest-adjacent
record rather than exposed as a user reloption.

For each routing level `L > 0`, store:

- `level`
- `fanout` or target child count used to train that level
- `nprobe_default` for routing from that level to level `L - 1`
- `sample_size` or effective sample bound
- `training_iterations`
- `centroid_dimensions`
- distance/operator family semantics, initially inner product to match the
  existing `ec_spire` scan path
- assignment payload format for the eventual leaves

Initial user-facing reloptions can stay single-value for compatibility:
`nlists` applies to leaf fanout and `nprobe` applies to level 1 unless
explicit per-level metadata exists. Recursive implementation should resolve
scan fanout in this order:

1. Per-level stored or reloption value for level `L`.
2. Current relation/session `nprobe` fallback for level 1.
3. A conservative default of one child for higher routing levels until a
   per-level control surface lands.

The hierarchy diagnostic should keep reporting whether per-level `nprobe` is
actually supported, not only whether the stored shape could support it.

## Recursive Build

Recursive build is a bottom-up coordinator over the existing single-level
training and leaf-object writer boundaries.

1. Collect and normalize heap source vectors using the existing SPIRE build
   path.
2. Train the level-1 leaf clustering over source vectors.
3. Route source rows into V2 leaf partition objects exactly as the single-level
   foundation does today.
4. Materialize one centroid record for each level-1 child PID.
5. If the number of centroid records fits under the configured root fanout, make
   one root routing object at level 1 and stop.
6. Otherwise, train the next level over the previous level's centroid records.
7. Allocate internal routing PIDs for clusters of lower-level routing objects,
   store their child PIDs and materialized centroids, and emit centroid records
   for those internal objects.
8. Repeat until one root routing object can cover the top-level centroid
   records.
9. Write all routing, leaf, placement, and manifest entries, validate the draft,
   then publish through the existing root/control epoch advance path.

The recursive coordinator should keep the current single-level build as a
degenerate case. If the configured target depth is one, or the trained
centroid count does not require another level, the bytes and diagnostics should
remain compatible with the current root-to-leaf hierarchy.

## Centroid Materialization

Centroids are not only transient training output. Phase 3 needs a materialized
centroid view so rebuild, diagnostics, update planning, and scans can inspect
the hierarchy without retraining.

The authoritative centroid for a child remains the centroid stored in its
parent routing object. A materialized centroid record is an indexable/readable
projection with:

```text
epoch
level
parent_pid
child_pid
centroid_ordinal
dimensions
centroid bytes
source_count
```

For the first implementation, this projection may be reconstructed from active
routing objects when diagnostics or tests need it. Persisting a separate
centroid object is reserved for the slice that needs rebuild/update mechanics
to read centroid groups without scanning every routing object.

## Scan Routing

Recursive scan descends level by level before leaf scoring:

1. Load the active epoch and the unique available root routing object.
2. Start with the root as the only current routing frontier.
3. At routing level `L`, score the query against each current parent object's
   child centroids.
4. Select top children using the same deterministic ordering as the current
   route heap: higher inner product, lower centroid ordinal, lower child PID.
5. If `L > 1`, load the selected internal children and repeat with `L - 1`.
6. If `L = 1`, treat selected children as leaf PIDs and run the existing leaf
   candidate scoring, dedupe, and exact rerank path.

The first level-local primitive should be pure and testable:

```text
route_level(parent_routing_object, query_vector, nprobe) -> selected child PIDs
```

The multi-level scan coordinator composes that primitive with snapshot child
loading and shape validation. It must preserve strict/degraded placement
semantics from the single-level scan path. Boundary-replica dedupe remains off
for the primary-only Phase 3 path unless a later phase explicitly enables it.

## Update Mechanics Interaction

Phase 2 split/merge rewrites one parent routing object in the single-level
foundation. With recursion, the same rule applies locally, then propagates
upward:

- A leaf split or merge rewrites its immediate level-1 parent.
- If the parent routing object's centroid changes at its own parent level, the
  parent may need a new replacement PID and an ancestor rewrite.
- Cross-parent merge is not leaf-local. It is a multi-parent coverage rewrite
  and must allocate replacement PIDs for every affected parent subtree.

Phase 3 should not attempt full recursive update propagation before recursive
build and scan are stable. Manual split/merge scheduling can remain limited to
single-level active hierarchies until the recursive update rules have separate
coverage.

## Deferred

The following work stays outside this Phase 3 design checkpoint:

- boundary replication and multi-PID vector membership fanout
- top-level graph routing over centroids
- local multi-store or remote placement
- background maintenance scheduling
- old-epoch physical tuple/page reclamation
- recursive split/merge propagation beyond single-parent local rules
- product-scale recall, latency, or storage measurements
- PG17 validation

## Review Checkpoint

The matching review packet is
`review/30469-spire-recursive-hierarchy-design/`. It asks for review of this
design boundary before recursive metadata or build code lands.
