# Review Request: Boundary Replica Bridge

## Scope

This packet continues Task 79 after the accepted candidate-surface baseline.
It adds a RaBitQ-primary `ecaz bench suite` bridge matrix for two questions from
packet 001:

- Does the missing `nlists=256` geometry point provide a useful Pareto bridge?
- Can boundary replicas recover high recall on smaller leaves while preserving
  the candidate gate?

No production scan behavior changed in this packet.

## Evidence

- Task definition: `plan/tasks/79-spire-candidate-surface-reduction.md`
- Suite config: `reviews/task-79/002-boundary-replica-bridge/suite-rabitq-boundary-bridge.json`
- Artifact manifest: `reviews/task-79/002-boundary-replica-bridge/artifacts/manifest.md`
- Suite status: `reviews/task-79/002-boundary-replica-bridge/artifacts/suite-status.log`
- Parsed report: `reviews/task-79/002-boundary-replica-bridge/artifacts/suite-report.md`
- Normalized rows: `reviews/task-79/002-boundary-replica-bridge/artifacts/results.jsonl`

## Result

The `nlists=256` bridge improves recall over `nlists=512`, but only by spending
too many candidates:

| nlists | fanout | boundary replicas | nprobe | candidates | p50 | recall@10 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 8 | 0 | 96 | 15,506,227 | 61.234 ms | 0.9975 |
| 256 | 16 | 0 | 96 | 7,582,639 | 37.339 ms | 0.9805 |
| 256 | 16 | 0 | 128 | 10,072,003 | 46.143 ms | 0.9910 |
| 512 | 16 | 1 | 96 | 8,042,299 | 51.444 ms | 0.9750 |
| 512 | 16 | 2 | 96 | 12,110,960 | 67.142 ms | 0.9845 |
| 1024 | 32 | 1 | 96 | 4,300,115 | 55.114 ms | 0.9635 |

No row satisfies both Task 79 gates:

- recall within `0.5 pp` of `0.9975`
- candidates `<=5.2M` over 200 queries

Boundary replicas are not the answer on this fixture. They recover some recall,
but by increasing candidates, p50 latency, and build cost.

## Review Focus

Please review whether this packet correctly answers the packet 001 feedback
and whether the resulting direction is sound: Task 79 should move away from
simple geometry/replica tuning and into a row-surface reduction mechanism that
does not score every row in selected high-recall leaves.

The proposed next implementation slice is a narrow design/code slice for either:

- row-budgeted routing with explicit selected-row estimates, if row-count
  metadata can be read without defeating the candidate-count win; or
- leaf-local subpartition pruning, if preserving recall requires selecting
  high-recall leaves but pruning rows inside each selected leaf.

## Validation

- `jq empty reviews/task-79/002-boundary-replica-bridge/suite-rabitq-boundary-bridge.json`
- `target/debug/ecaz ... bench suite audit --config reviews/task-79/002-boundary-replica-bridge/suite-rabitq-boundary-bridge.json`
- `target/debug/ecaz ... bench suite run --dry-run --config reviews/task-79/002-boundary-replica-bridge/suite-rabitq-boundary-bridge.json`
- `target/debug/ecaz ... bench suite run --config reviews/task-79/002-boundary-replica-bridge/suite-rabitq-boundary-bridge.json`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/002-boundary-replica-bridge/artifacts/suite-manifest.json`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/002-boundary-replica-bridge/artifacts/suite-manifest.json`
