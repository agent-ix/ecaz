# Review Request: Unseeded Graph Linear Fallback

Commit: `5280f65`

Scope:
- `src/am/scan.rs`

Summary:
- record whether `amrescan` successfully seeded any graph traversal candidates into the
  graph/search-owned visible frontier
- narrow the linear fallback shell so it remains available only when graph traversal never seeded
  at all, instead of remaining available for any scan that has not yet materialized a graph-ordered
  result
- keep the reset path explicit by clearing the seeded state on rescan before rebuilding scan
  traversal state

Please review:
- whether `graph_traversal_seeded` is the right state boundary for deciding when linear fallback is
  still allowed
- whether `seed_bootstrap_trace(...)` is the correct and only place that should mark graph
  traversal as seeded
- whether the updated unit tests capture the intended A3 contract: once graph traversal starts, it
  owns the scan to exhaustion
