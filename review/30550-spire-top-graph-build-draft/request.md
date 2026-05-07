# Review Request: SPIRE Top-Graph Build Draft

- Code commits:
  - `014cb947` (`Build SPIRE top graph drafts`)
  - `c48a96df` (`Build SPIRE top graphs from routing roots`)
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

This is intentionally still in-memory build plumbing. It does not persist graph
object bytes, add reloptions, or replace scan routing yet.

## Files

- `src/am/ec_spire/build/top_graph.rs`
- `src/am/ec_spire/build/tests/top_graph.rs`
- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/build/tests.rs`

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

## Validation

- `cargo test --lib top_graph --no-default-features --features pg18`
- `git diff --check`

`cargo fmt --check` was also attempted, but it reports an unrelated pre-existing
formatting diff in `src/am/ec_ivf/scan.rs`; this checkpoint leaves that file
unchanged.
