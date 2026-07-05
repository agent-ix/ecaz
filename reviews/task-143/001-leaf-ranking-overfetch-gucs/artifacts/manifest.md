# Task 143 Packet 001 Artifact Manifest

- Head SHA: `8ad0b0d7c81bfd610503a60b0abb42255d9ac10d`
- Branch: `task-143-spire-leaf-ranking-route-overfetch`
- Task bucket: `reviews/task-143/001-leaf-ranking-overfetch-gucs`
- Slice: default-off leaf-score-only route ranking plus recursive route overfetch budget plumbing.
- Fixture / storage / rerank mode: Rust unit-level routing and option resolver tests only; no corpus fixture, storage format, rerank mode, or benchmark suite in this slice.
- Isolated/shared surface: not applicable; no PostgreSQL or benchmark table surface was created.

## Artifacts

| Artifact | Command | Timestamp | Key result |
| --- | --- | --- | --- |
| `artifacts/cargo-test-recursive-route-budget.log` | `cargo test -p ecaz recursive_route_budget --no-default-features --features pg18` | 2026-07-05 06:24:50-07:00 | `2 passed; 0 failed; 2261 filtered out` |
| `artifacts/cargo-test-leaf-score-routing.log` | `cargo test -p ecaz route_recursive_routing_objects_to_leaf_routes_can_rank_final_leaves_by_leaf_score_only --no-default-features --features pg18` | 2026-07-05 06:24:54-07:00 | `1 passed; 0 failed; 2262 filtered out` |

## Scope Notes

This packet is the Task 143 code-plumbing checkpoint. It does not claim the Task 143 release A/B acceptance matrix. Later packets still need `ecaz bench suite` release evidence for leaf-only ranking and route-overfetch alpha sweeps at 10k / 50k / 100k, including the route-containment funnel required by the task.
