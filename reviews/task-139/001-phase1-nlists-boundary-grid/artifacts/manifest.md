# Task 139 Packet 001 Taint Annotation

- Task bucket: `reviews/task-139/001-phase1-nlists-boundary-grid/`
- Annotation timestamp: `2026-07-04`
- Annotation commit: `6926269716f7ed8a846247c4314c588a95707aed`
- Superseding packet: `reviews/task-141/001-release-anchor-rebaseline/`

## Status

Task 139 phase 1 is superseded by the Task 141 bench-integrity program. Do not restart or extend the Task 139 cells.

## Taint

The Task 139 phase-1 multinode latency grid used the pre-Task-141 local multinode substrate. That substrate installed debug-profile `ecaz.so` on the coordinator and workers, while the suite release guard did not cover the `spire-pipeline` step kind. As a result, Task 139 latency rows are debug-build latency evidence and must not be cited as release SPIRE performance.

## Replacement Evidence

Task 141 packet 001 provides release-profile multinode anchors with per-node backend provenance:

- `reviews/task-141/001-release-anchor-rebaseline/artifacts/manifest.md`
- `reviews/task-141/001-release-anchor-rebaseline/artifacts/release-50k-n128-b0-r2/bench-suite/results.jsonl`
- `reviews/task-141/001-release-anchor-rebaseline/artifacts/release-50k-n1024-b0-r2/bench-suite/results.jsonl`
- `reviews/task-141/001-release-anchor-rebaseline/artifacts/release-100k-n1024-b0-r2/bench-suite/results.jsonl`
- `reviews/task-141/001-release-anchor-rebaseline/artifacts/debug-50k-n1024-b0-r2/bench-suite/results.jsonl`

The matched Task 141 50k n1024/b0 A/B shows debug query p50 is 5.27x to 5.73x slower than release with unchanged recall. This is the durable explanation for the Task 139 debug-grid latency regime.
