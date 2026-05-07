# SPIRE Top-Level Graph Design

Status: Phase 6 design checkpoint for Task 30
Date: 2026-05-07
Scope: local top-level graph routing over SPIRE recursive centroid objects

This note chooses the first top-level graph shape for SPIRE. It is subordinate
to ADR-049, the Phase 0 partition-object storage design, the Phase 3 recursive
hierarchy plan, the Phase 4 local placement plan, and the Phase 5 boundary
replication plan.

## Goals

Phase 6 replaces the current flat scan over the highest routing level with a
graph-assisted routing step while preserving the existing lower-level SPIRE
descent:

```text
query
  -> top-level graph over routing centroids
  -> selected top-level routing PIDs
  -> existing recursive route descent
  -> leaf scoring, vec_id dedupe, exact rerank
```

The first implementation must:

- keep the local strict epoch/placement model;
- preserve flat routing as a deterministic fallback and comparison path;
- avoid a generic graph abstraction before there are two SPIRE graph
  implementations;
- avoid creating a nested PostgreSQL `ec_diskann` or `ec_hnsw` index inside an
  `ec_spire` index;
- expose enough diagnostics to tell whether graph routing was used and how much
  fanout it returned.

This phase does not add remote placement, graph-aware maintenance rewrites,
background scheduling, or product-scale graph measurements.

## Decision

Use a single-layer Vamana/DiskANN-style graph over top-level SPIRE centroids for
the first SPIRE top-level graph.

Do not use HNSW for the first SPIRE graph checkpoint. The existing HNSW code is
an access-method lifecycle with heap-row graph nodes, level metadata, live
insert, and vacuum behavior that is not the shape of a SPIRE root-routing
accelerator. A multi-layer HNSW graph also adds another level hierarchy on top
of SPIRE's own recursive hierarchy.

Do not make the graph algorithm build-time selectable yet. A selectable
`spire_top_graph_algorithm` reloption would force a storage and diagnostics
abstraction before SPIRE has a second graph implementation. Keep the bytes and
routing contract concrete first.

## Why Vamana

The `ec_diskann` lane already has a pure in-memory Vamana graph core:

```text
VamanaGraph
build_vamana_graph_with_stats
greedy_search
```

That code is closer to the SPIRE need than the current AM-specific graph
storage:

- it operates on dense node IDs, independent of heap TIDs;
- it accepts caller-supplied distance functions;
- it is single-layer, matching SPIRE's "top-level accelerator" role;
- it already has build statistics that can be surfaced in SPIRE diagnostics;
- its graph nodes can map cleanly to top-level routing child PIDs.

The SPIRE integration should reuse the pure algorithmic core, not the
`ec_diskann` AM page format, callbacks, reloptions, or mutation lifecycle.

## Graph Node Model

A top-graph node represents one top-level routing child, not one heap vector and
not one leaf assignment row.

```text
node_id: dense u32 graph-local ordinal
child_pid: SPIRE routing PID selected by the top-level graph
centroid_ordinal: ordinal from the graph parent/root centroid list
centroid: f32 centroid vector used for graph build and query distance
neighbors: dense u32 graph-local neighbor ordinals
```

The graph is an epoch object. It is compatible with exactly the routing
centroid set that was published in the same epoch. Scans must not combine a top
graph from one epoch with routing objects from another.

For the first local implementation, graph build and search should score
centroid-to-centroid and query-to-centroid distances with full `f32` inner
product over the materialized centroids. Top-level centroid count is far smaller
than heap vector count, so this is simpler and more stable than using a
quantized graph payload before measurements require it.

## Placement and Persistence

Persist the top graph as a SPIRE partition object referenced by the epoch
manifest, not as a child PostgreSQL index.

The object should be placement-routed like any other SPIRE object. For the first
local implementation, storing the graph in the same store selected by its graph
object PID is sufficient; remote replication and graph-object availability are
Phase 7 concerns.

The first byte format can be one graph object with:

```text
header.kind = TopGraph
header.level = root_level
root_pid
dimensions
node_count
graph_degree
build_list_size
alpha
entry_node
repeated node record:
  child_pid
  centroid_ordinal
  neighbor_count
  neighbor_ordinals[graph_degree]
```

Centroid vectors do not need to be duplicated in the graph object if they remain
authoritatively available from the root/top routing object. The graph object can
store only ordinals and child PIDs, and the scan loader can join nodes back to
the routing centroid list. If loading cost becomes an issue, duplicate centroid
blocks in a later format version.

## Recursive Build Interaction

The current recursive coordinator compresses centroid records until one root
routing object has at most `recursive_fanout` children. A useful top graph needs
a larger top-level centroid set than that. Phase 6 therefore needs a distinct
build boundary:

1. Build leaf objects and lower internal routing objects as Phase 3 does.
2. Stop recursive compression at a configured top graph input level instead of
   always compressing until one small root covers all children.
3. Publish one root/top routing object that owns the top graph input child
   centroids.
4. Build the Vamana graph over those top routing children.
5. Publish the graph object in the same epoch manifest.

The first code slice may still build a graph over the existing small root child
set to validate storage and scan plumbing. That is an implementation proof, not
the final scale shape. The design target is a root/top routing object whose
child set is graph-searchable rather than flat-scanned for large recursive
indexes.

## Scan Routing

Graph-assisted scan is a top-level replacement only.

1. Load the active root/top routing object and its matching top graph object.
2. Run Vamana greedy search from the graph entry node using query-to-centroid
   distance.
3. Select a bounded top-graph frontier.
4. Map selected graph node ordinals to child PIDs from the root/top routing
   object.
5. Continue with the existing recursive descent from those selected routing
   PIDs.
6. At level 1, keep the existing `nprobe` leaf fanout, candidate scoring,
   boundary-replica dedupe, and exact rerank behavior.

The route order exposed to lower levels must be deterministic. For equal graph
distances, tie-break by lower centroid ordinal, then lower child PID, then lower
graph node ordinal.

Flat root routing remains the fallback when:

- no graph object exists in the active epoch;
- the active graph object is marked unavailable in degraded mode and strict
  mode is not requested;
- the graph has too few nodes for the configured graph threshold;
- the scan path explicitly requests flat comparison diagnostics.

Strict mode should fail closed for malformed graph objects, graph/root child
set mismatch, missing selected child PID, or impossible entry node.

## Configuration

Keep Phase 6 configuration narrow and explicit:

```text
top_graph_enabled bool          -- default false for the first landing slice
top_graph_degree int            -- default 32, bounded
top_graph_build_list_size int   -- default 100, bounded
top_graph_alpha real            -- default 1.2
top_graph_search_list_size int  -- default from nprobe or a small fixed bound
```

Only add these reloptions when the implementation needs them. The first
design/primitive slice can use constants in pure tests. Do not add a
`top_graph_algorithm` reloption until a second graph algorithm exists.

## Diagnostics

Expose graph status through SQL diagnostics before making measurement claims:

```text
top_graph_enabled
top_graph_present
top_graph_algorithm = 'vamana'
top_graph_node_count
top_graph_degree
top_graph_entry_node
top_graph_object_bytes
top_graph_route_count
top_graph_fallback_reason
top_graph_build_stats
```

For scan diagnostics, report whether the query used graph or flat top routing,
how many graph nodes were visited, how many top-level child PIDs were selected,
and how many leaf PIDs were finally scored.

## Measurement Gate

The first packet after implementation should compare graph routing against flat
top-level routing on the same recursive real-corpus fixture:

- same corpus, storage format, local-store layout, boundary setting, and leaf
  `nprobe`;
- flat top routing versus Vamana top graph;
- recall@10, mean/p50/p95 query time, graph visited count, selected top PIDs,
  scored leaf count, and graph object bytes.

Small local fixtures are acceptable for correctness and routing-shape evidence.
Product claims require a larger recursive hierarchy where flat top routing is
actually non-trivial.

## Deferred

- HNSW top graph.
- Build-time graph algorithm selection.
- Graph-aware split/merge maintenance propagation.
- Remote graph object placement or graph replicas.
- Quantized top-graph centroid payloads.
- Graph object physical reclamation beyond existing epoch cleanup policy.

## Review Checkpoint

The matching review packet is
`review/30549-spire-top-level-graph-design/`. It asks for review of the graph
choice and the persistence/routing boundary before graph object bytes land.
