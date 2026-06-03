# Review Request: Candidate Surface Baseline

## Scope

This packet starts Task 79 with a RaBitQ-primary `ecaz bench suite` geometry
matrix. It includes the task definition, task-index update, suite config, and
packet-local artifacts for the first candidate-surface measurement pass.

No production scan behavior changed in this packet.

## Evidence

- Task definition: `plan/tasks/79-spire-candidate-surface-reduction.md`
- Task index: `plan/tasks/README.md`
- Suite config: `reviews/task-79/001-candidate-surface-baseline/suite-rabitq-geometry.json`
- Artifact manifest: `reviews/task-79/001-candidate-surface-baseline/artifacts/manifest.md`
- Suite status: `reviews/task-79/001-candidate-surface-baseline/artifacts/suite-status.log`
- Parsed report: `reviews/task-79/001-candidate-surface-baseline/artifacts/suite-report.md`
- Normalized rows: `reviews/task-79/001-candidate-surface-baseline/artifacts/results.jsonl`

## Result

The exact Task 78-style baseline reproduced the problem:

- `nlists=128`, `recursive_fanout=8`, `top_graph_search_list_size=96`, `nprobe=96`
- `candidate_sum=15,506,227`
- `route_sum=19,200`
- recall@10 `0.9975`
- p50 `61.234 ms`

Increasing leaf density directly reduced candidates, but not at matched recall:

| nlists | fanout | nprobe | candidates | p50 | recall@10 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 8 | 96 | 15,506,227 | 61.234 ms | 0.9975 |
| 512 | 16 | 96 | 4,008,683 | 35.474 ms | 0.9420 |
| 512 | 16 | 128 | 5,337,119 | 40.210 ms | 0.9645 |
| 1024 | 32 | 96 | 2,152,562 | 46.472 ms | 0.9265 |
| 2048 | 64 | 96 | 1,148,089 | 76.914 ms | 0.8875 |

## Review Focus

Please review whether Task 79 now directly targets the candidate-count problem
from Tasks 75-78 and whether this first packet supports the Phase 1 decision:
geometry alone cuts the scored row surface, but does not meet the high-recall
candidate gate.

The proposed next slice is not another heap cutoff. It should address
accuracy-preserving candidate selection: better recursive/top-graph routing at
lower row surfaces, row-budgeted route selection, or leaf-local pruning that
preserves the current high-recall leaf coverage without scoring every row.

## Validation

- `jq empty reviews/task-79/001-candidate-surface-baseline/suite-rabitq-geometry.json`
- `git diff --check`
- `target/debug/ecaz ... bench suite audit --config reviews/task-79/001-candidate-surface-baseline/suite-rabitq-geometry.json`
- `target/debug/ecaz ... bench suite run --dry-run --config reviews/task-79/001-candidate-surface-baseline/suite-rabitq-geometry.json`
- `target/debug/ecaz ... bench suite run --config reviews/task-79/001-candidate-surface-baseline/suite-rabitq-geometry.json`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/001-candidate-surface-baseline/artifacts/suite-manifest.json`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/001-candidate-surface-baseline/artifacts/suite-manifest.json`
