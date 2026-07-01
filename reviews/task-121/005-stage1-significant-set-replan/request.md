# Review Request: Task 121 Stage 1 Significant Set And Phase 2 Re-Plan

## Scope

This packet is the Task 121 Phase 1 boundary review. It does not add new code or new measurements. It distills the completed local 100k OFAT screen in `reviews/task-121/001-stage1-routing-screen/` into a significant-lever set and proposes the Phase 2 local factorial grid.

Reviewer sign-off is requested before Phase 2 benches, matching the task's re-plan gate.

## Stage 1 Decision

The Stage 1 funnel result is stable across the screen: route-stage containment equals final recall in every completed run. The route stage is still the bottleneck. Later placement/candidate/rerank stages are not the source of observed recall loss in this screen.

Significant set for Phase 2:

| Lever | Carry forward? | Reason |
| --- | --- | --- |
| `boundary_replica_count` | Yes, primary | Strongest fixed-nprobe route-containment gain. Bound1, bound2, and bound4 all improve recall, with a clear cost/benefit knee to map. |
| `training_sample_rows=50000` | Yes, secondary | Positive at low/mid nprobe, cheap, and better than `training_sample_rows=100000` on this fixture. |
| `nlists=316` | Yes, interaction/cost axis only | Not a standalone route-recall winner, but materially reduces candidate volume/latency and may pair well with boundary replication. |
| `top_graph_search_list_size=200` | No, diagnostic only | High-nprobe correction reaches perfect recall by scanning the full 100k corpus per query. That proves the ceiling, not a practical lever. |
| `recursive_fanout` | No | Fanout16 has a small non-monotonic gain; fanout32 does not confirm a meaningful trend. |
| `top_graph_degree` | No | Degree48 and degree64 exactly match baseline route containment, final recall, and candidate volume. |
| `storage_format=turboquant` | No for Phase 2 route factorial | Recall-neutral versus RaBitQ; keep it for compatibility/Pareto follow-up, not route recovery. |

## Proposed Phase 2 Grid

Run local-only `ecaz bench suite` factorial drills at `10k`, `50k`, and `100k`.

Axes:

- `boundary_replica_count`: `0, 1, 2, 4`
- `training_sample_rows`: `10000, 50000`
- `nlists`: `128, 316`
- `storage_format`: `rabitq`

Sweep:

- `nprobe`: `4, 8, 12, 16, 24, 32, 48, 64, 96`
- Extend above `96` only for rows that have not reached a route-containment/recall knee.

Metrics:

- route-stage containment from the funnel;
- final recall@10;
- candidate volume;
- latency;
- storage.

## Explicit Non-Scope

- No AWS.
- No PQ.
- No Phase 3 scan-efficiency levers until Phase 2 identifies a recall-recovering config.
- No task-doc update in this packet; reviewer is handling task-doc updates.

## Evidence

See `artifacts/manifest.md`. The evidence is the completed Stage 1 packet-local benchmark set under `reviews/task-121/001-stage1-routing-screen/`.
