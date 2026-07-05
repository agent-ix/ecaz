# Task 121 Stage 1 Significant-Set Replan Manifest

- head_sha: 8157187b5deb97c418ca5a5e33295a98009de043
- task_bucket: reviews/task-121
- packet: reviews/task-121/005-stage1-significant-set-replan
- lane: intel-local
- fixture: derived review/re-plan packet over the completed 100k Stage 1 screen in `reviews/task-121/001-stage1-routing-screen`
- storage formats considered: RaBitQ baseline and TurboQuant control only
- AWS: not used
- timestamp: 2026-06-23 America/Los_Angeles

## Source Evidence

This packet introduces no new benchmark run. It cites the completed Stage 1 packet:

- `reviews/task-121/001-stage1-routing-screen/request.md`
- `reviews/task-121/001-stage1-routing-screen/artifacts/manifest.md`

The Stage 1 source packet contains packet-local `ecaz bench suite` configs, suite manifests, result JSONL files, pipeline logs, funnel JSONL, stage-containment JSONL, route-containment TSVs, and storage/load logs for the completed 100k screen.

## Derived Significant Set

| Lever | Stage 1 decision | Source |
| --- | --- | --- |
| `boundary_replica_count` | Primary significant route-containment lever. Bound1, bound2, and bound4 all improve route containment/final recall at fixed nprobe, with diminishing returns and steep cost. | `request.md` sections "Second/Fourth OFAT lever result" |
| `training_sample_rows=50000` | Secondary low/mid-nprobe route lever. Better than 100k on this fixture and cheaper than boundary replication. | `request.md` sections "Twelfth/Thirteenth OFAT lever result" |
| `nlists=316` | Not significant alone for route recall, but useful as a cost/interaction axis when paired with boundary replication. | `request.md` sections "Fifth/Seventh OFAT lever result" |
| `top_graph_search_list_size=200` | Diagnostic ceiling only. Perfect recall requires full-corpus fanout at high nprobe. | `request.md` section "Corrected high-nprobe tgsl200 screen" |
| `recursive_fanout` | Not significant. Fanout16 has a small non-monotonic gain; fanout32 fails to confirm the trend. | `request.md` sections "Eighth/Ninth OFAT lever result" |
| `top_graph_degree` | Negative. Degree48 and degree64 exactly match baseline recall/candidates. | `request.md` sections "Tenth/Eleventh OFAT lever result" |
| `storage_format=turboquant` | Recall-neutral storage-format control, not a route-recall lever. | `request.md` section "Fourteenth OFAT lever result" |

## Proposed Phase 2 Local Grid

Run local-only factorial drills at 10k, 50k, and 100k. Use `ecaz bench suite`; no AWS and no PQ.

Primary grid:

- `boundary_replica_count`: `0, 1, 2, 4`
- `training_sample_rows`: `10000, 50000`
- `nlists`: `128, 316`
- `storage_format`: `rabitq`

Sweep:

- `nprobe`: `4, 8, 12, 16, 24, 32, 48, 64, 96`
- Add higher nprobe only when the row has not reached a visible recall/route-containment knee by 96.

Reasoning:

- Boundary replication is the only strong route-stage recovery lever from Stage 1.
- Train50k is cheap and positive at low/mid nprobe, so it should be tested for interaction with boundary replication.
- `nlists=316` hurt recall alone but cut candidate rows materially; paired with boundary replication it may recover recall at a lower candidate/latency point.
- TurboQuant is held out of the route-factorial grid because Stage 1 showed it is route/recall neutral. It can be reintroduced in Phase 3 scan-efficiency/Pareto if a recall-recovering config exists.

Reviewer sign-off requested before running this Phase 2 matrix, per the task re-plan gate.
