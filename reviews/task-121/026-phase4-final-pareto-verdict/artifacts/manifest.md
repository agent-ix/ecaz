# Task 121 Packet 026 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/026-phase4-final-pareto-verdict/`
- Head SHA: `10d62da1934d46f553e5200bdbe0cc68a624911e`
- Packet manifest written: `2026-06-26T02:35:29Z`
- Lane: local review synthesis, no new benchmark execution
- Status: final Phase 4 Pareto/verdict packet

## Evidence Sources

- `reviews/task-121/001-stage1-routing-screen/`: Phase 1 OFAT route screen.
- `reviews/task-121/005-stage1-significant-set-replan/`: significant-lever
  set and Phase 2 re-plan.
- `reviews/task-121/011-phase2-local-10k-axis-fix-run/`: 10k factorial slice.
- `reviews/task-121/016-phase2-local-50k-b2-b4-f8-run/`: 50k b2/b4 f8 slice.
- `reviews/task-121/017-phase2-local-100k-axis-fix-run/`: 100k factorial
  axis-fix matrix.
- `reviews/task-121/018-phase2-local-100k-b8-latency-followup/`: 100k b8
  wall and clean-latency follow-up.
- `reviews/task-121/019-phase3-local-rabitq-sampled-pruning/`: 10k
  block-pruning pilot.
- `reviews/task-121/022-phase3-local-50k-retuned-latency-pipeline/`: 50k
  retuned block-pruning latency/pipeline.
- `reviews/task-121/024-phase3-local-100k-retuned-latency-pipeline/`: 100k
  retuned block-pruning latency/pipeline.
- `reviews/task-121/025-phase3-turboquant-block-summary-decision/`: TurboQuant
  block-summary implementation-gap decision.

## No New Runs

This packet is a synthesis of already committed packet-local evidence. It does
not add benchmark logs beyond this manifest and the review request.

## Verdict

Do not promote a new default from Task 121. Local routing precision can be
recovered, but the recovery mechanism is boundary replication, and its storage
and latency slope is too steep for a default. The best named follow-up
candidate is `b4/tr50/f8` RaBitQ, with retuned sampled block pruning enabled
only for high-recall `nprobe=96` experiments. The evidence-backed wall is that
the route-stage loss can be bought back, but not cheaply enough at the likely
low operating point.
