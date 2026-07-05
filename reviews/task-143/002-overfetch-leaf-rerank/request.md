# Review Request: Task 143 Packet 002 — Overfetch Leaf Rerank

## Summary

This slice makes Task 143 route overfetch semantically active.

Packet 001 widened `SpireRecursiveRouteBudget`, but final selection still stopped at the selected-leaf cap. This patch changes leaf selection to:

1. Deduplicate and collect route-ordered leaf routes up to `max_leaf_routes` (`α * nprobe`).
2. Rerank the overfetched cushion by exact leaf score.
3. Truncate back to `selected_leaf_routes` / effective `nprobe`.

`SpireRecursiveLeafRoute` now carries `leaf_score` separately from the visible `route_score`, so accumulated-score routing can overfetch by route score and still rerank by leaf score before final trimming.

## Code Under Review

- `src/am/ec_spire/scan/types.rs`: add `leaf_score` to `SpireRecursiveLeafRoute`.
- `src/am/ec_spire/scan/routing.rs`: collect overfetched routes, rerank by leaf score, truncate to selected leaf cap.
- `src/am/ec_spire/scan/{candidates,snapshot}.rs`: populate `leaf_score` in non-recursive route construction.
- Tests add `route_recursive_routing_objects_to_leaf_routes_overfetch_reranks_by_leaf_score`.

## Validation

- `artifacts/cargo-test-routing-overfetch-rerank.log`
  - `cargo test -p ecaz route_recursive_routing_objects_to_leaf_routes_ --no-default-features --features pg18`
  - Result: `4 passed; 0 failed`.

## Review Focus

1. Confirm α-overfetch now actually considers up to `max_leaf_routes` unique leaves before final trimming.
2. Confirm final trimming uses leaf-score rerank and still returns no more than effective `nprobe`.
3. Confirm default α=1 remains behavior-compatible.
4. Confirm this code path is ready for the release `ecaz bench suite` A/B packet.
