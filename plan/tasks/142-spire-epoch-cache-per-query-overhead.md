# Task 142: SPIRE Epoch-Keyed Caching — Eliminate the O(nlists) Per-Query Tax

Status: proposed (2026-07-04; remediation program task 2 of 6).
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 — pure latency win, no recall trade; de-taxes the high-nlists
geometry every later task depends on. Depends on Task 141 (measurable
substrate).

## Why

The coordinator does up to 5–6 full O(nlists) manifest walks per query, each
doing a buffer-pinned tuple read + header decode per partition, with no
cross-query cache anywhere. Measured signature (Task 139 grid): a fixed
per-query gap of ~41/137/269 ms at nlists 128/512/1024 — linear in nlists,
flat in nprobe (debug-inflated in absolute terms, but the mechanism is real).

The walks:

1. Plan time, before any profiler starts:
   - `ec_spire_amcostestimate` (`src/am/ec_spire/cost/mod.rs:63-92`,
     `compute_amcostestimate` `:198-220`) does two independent full walks:
     `cost_active_snapshot_diagnostics` →
     `collect_snapshot_diagnostics` (`diagnostics/mod.rs:79-190`,
     `read_object_header` per placement entry), and
     `cost_index_hierarchy_snapshot`
     (`coordinator/hierarchy_snapshots.rs:1629-1720`) re-deserializing all
     routing objects/centroids for summary stats.
   - `ec_spire_amgettreeheight` (`cost/mod.rs:238-245`) does a third full
     walk to return an i32 depth.
2. Inside the profiled `planning` phase (`scan_output.rs:554-695`):
   - `count_scan_plan_routable_leaf_pids` loads the entire routing hierarchy
     (`scan/routing.rs:239-246` →
     `load_snapshot_coordinator_routing_hierarchy` `:74-138`) just to count
     leaves, then discards it.
   - `collect_scan_plan_selected_leaf_pids` loads it again (`routing.rs:210`)
     plus a full header walk to locate the top-graph object
     (`routing.rs:167-196`).
3. Remote side: every `ec_spire_remote_search*` invocation re-opens the
   index, re-reads all three manifest blobs, re-deserializes the placement
   directory (O(nlists×nodes), `meta/placement_directory.rs:40-83`), and
   re-opens the object store set (`coordinator/hierarchy_snapshots.rs:359-435`).
4. Per-connection overheads on the pooled async path: regclass probe +
   endpoint-identity query every query (the identity cache in
   `governance.rs:286-355` is wired only into the legacy sync executor, not
   `dispatch.rs:1295-1338`); 2 advisory-lock SPI round trips per node per
   query (`governance.rs:85-91,138-148,182-219`).

All of this is epoch-stable: it changes only when a new epoch is published.
The actual routing descent is sub-millisecond.

## Goal

One routing-hierarchy load per query at most, everything epoch-stable cached
per backend keyed on `(index_oid, active_epoch)`, cost callbacks reading
publish-time statistics instead of walking the index, remote sessions reusing
their snapshot. Fixed per-query overhead should stop scaling with nlists.

## Scope

### Phase 0 — Instrument first

- Split the `planning` phase into `manifest_load` / `leaf_count` /
  `route_select` / `local_heap` sub-phase timers in
  `SpireRemoteProductionReadMetrics`.
- Time `compute_amcostestimate` / `index_hierarchy_snapshot` /
  each `load_snapshot_coordinator_routing_hierarchy` call; confirm the
  staircase via EXPLAIN-only latency vs nlists on release build.

### Phase 1 — Coordinator caching

- Backend-local epoch-keyed cache for routing hierarchy, manifests, and
  top-graph locator; share one load across leaf-count / route-select /
  top-graph within a query.
- Publish-time cost statistics (leaf_assignment_count, routing_object_bytes,
  hierarchy depth, centroid dims) stored in the epoch manifest; cost
  callbacks read them instead of walking objects.

### Phase 2 — Remote/session + transport hygiene

- Remote-side per-session epoch snapshot cache (manifests, placement
  directory, object store set).
- Wire the endpoint-identity cache into the async production path; drop the
  per-query regclass probe on pooled connections; evaluate replacing the
  per-query SPI advisory-lock pair with a session-scoped permit.

### Phase 3 — A/B

- Release-build A/B at 50k/100k on nlists {128, 1024, 2048}: fixed per-query
  overhead before/after, EXPLAIN-only time, end-to-end p50/p95, recall
  unchanged (bit-identical results expected).

## Required Evidence

- `ecaz bench suite` A/B per the closeout rule (10k/50k/100k for the
  affected surface); sub-phase timer tables in the packet; identical result
  IDs pre/post.

## Non-Goals

- No routing algorithm changes (Task 143/144). Cache invalidation must key
  strictly on published epoch — no weakening of snapshot semantics.

## Acceptance Criteria

1. Sub-phase instrumentation landed; the nlists-linear staircase reproduced
   on release, then eliminated (fixed overhead flat in nlists within noise).
2. At most one routing-hierarchy disk load per query; cost callbacks do zero
   per-query O(nlists) walks.
3. Remote invocations reuse session snapshot state.
4. A/B evidence at 10k/50k/100k, recall/results unchanged.

## References

- `src/am/ec_spire/cost/mod.rs`, `src/am/ec_spire/scan/routing.rs:74-246`,
  `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs:554-695`,
  `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`,
  `src/am/ec_spire/coordinator/remote_candidates/governance.rs`,
  `src/am/ec_spire/meta/placement_directory.rs:40-83`
- `plan/tasks/125-spire-distributed-read-transport-efficiency.md` (gated
  predecessor; this task subsumes its overhead-profiling intent)
