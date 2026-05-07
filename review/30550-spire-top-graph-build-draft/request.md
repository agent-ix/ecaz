# Review Request: SPIRE Top-Graph Build Draft

- Code commits:
  - `014cb947` (`Build SPIRE top graph drafts`)
  - `c48a96df` (`Build SPIRE top graphs from routing roots`)
  - `6623bc41` (`Route SPIRE top graphs over routing roots`)
  - `6feba1da` (`Add SPIRE top graph object codec`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 6 top-level graph
- Agent: coder1

## Summary

This checkpoint adds the first Phase 6 code primitive for a SPIRE top-level
graph:

- adds `SpireTopGraphBuildInput`, node input, node output, and build draft
  structs;
- validates top-graph root PID, dimensions, graph degree, build list size,
  alpha, child PID uniqueness, centroid ordinal uniqueness, centroid
  dimensionality, and finite centroid components;
- converts centroid inner product into a nonnegative build distance using a
  finite node-set offset, matching the Vamana core's nonnegative-distance
  requirement while preserving inner-product ordering;
- selects a deterministic approximate-medoid entry node;
- builds a Vamana graph through the existing pure `ec_diskann`
  `build_vamana_graph_with_stats` core;
- preserves graph-local node ordinals as the mapping from graph neighbors back
  to top routing child PIDs and centroid ordinals.
- adds a root-routing-object adapter that projects root child entries into the
  top-graph input model and rejects internal routing objects.
- adds a pure scan-side top-graph route helper that validates graph/root
  compatibility, runs Vamana greedy search, and returns deterministic selected
  child PIDs without wiring live scan callbacks yet.
- adds a durable `TopGraph` partition-object kind and V1 codec carrying root
  PID, dimensions, graph degree, build list size, alpha, entry node, child
  PIDs, centroid ordinals, and neighbor ordinals.

This still does not publish graph object bytes into live epochs, add reloptions,
or replace live scan routing yet.

## Files

- `src/am/ec_spire/build/top_graph.rs`
- `src/am/ec_spire/build/tests/top_graph.rs`
- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/build/tests.rs`
- `src/am/ec_spire/scan.rs`
- `src/am/ec_spire/scan/routing.rs`
- `src/am/ec_spire/scan/tests.rs`
- `src/am/ec_spire/scan/tests/routing.rs`
- `src/am/ec_spire/storage/top_graph.rs`
- `src/am/ec_spire/storage/tests/top_graph.rs`
- `src/am/ec_spire/storage/relation_plan.rs`
- `src/am/ec_spire/storage/vec_id.rs`

## Review Focus

1. Check the distance conversion from centroid inner product to nonnegative
   Vamana distance.
2. Confirm the draft shape has enough metadata for the planned graph object
   format: root PID, dimensions, node count, degree, build list size, alpha,
   entry node, child PIDs, centroid ordinals, neighbors, and Vamana stats.
3. Check whether rejecting duplicate child PIDs and duplicate centroid ordinals
   is the right validation boundary for joining graph nodes back to the root
   routing centroid set.
4. Confirm this is a reasonable first build-integration slice before durable
   graph object codecs and scan routing land.
5. Check whether the root-only adapter is the right boundary for the initial
   top graph, given that the design target is one graph over the root/top
   routing child set.
6. Review the scan-side graph route validation: root PID, dimensions, node
   count, node ordering, child PID/centroid ordinal join, entry node, neighbor
   bounds, and search-list versus route-count constraints.
7. Confirm deterministic route ordering should sort by graph distance,
   centroid ordinal, child PID, then graph node ordinal.
8. Review the `TopGraph` partition-object byte format and validation: root
   linkage via `parent_pid`, node count in `child_count`, entry-node bounds,
   duplicate child/centroid rejection, neighbor bounds, no self-neighbors, and
   neighbor-count <= graph degree.

## Validation

- `cargo test --lib top_graph --no-default-features --features pg18`
- `git diff --check`

`cargo fmt --check` was also attempted, but it reports an unrelated pre-existing
formatting diff in `src/am/ec_ivf/scan.rs`; this checkpoint leaves that file
unchanged.
