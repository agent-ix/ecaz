# Review Request: Rerank Budget Probe

## Scope

Task 79 RaBitQ-only measurement packet for the n512 / top-graph-256 row-budget surface. This packet checks whether increasing rerank width can recover recall while keeping the candidate surface and p50 latency near the Task 79 gates.

## Result

The rerank-width probe does not close Task 79. The fastest row reaches 43.643 ms p50, but recall is 0.9755. The higher-nprobe rows improve recall only to 0.9840, still below the 0.9925 floor.

| step | nprobe | candidates | routes | p50 ms | recall |
| --- | ---: | ---: | ---: | ---: | ---: |
| n512 row24k rerank50 | 160 | 4,831,812 | 23,222 | 45.279 | 0.9755 |
| n512 row24k rerank50 | 192 | 4,831,812 | 23,222 | 51.349 | 0.9840 |
| n512 row24k rerank100 | 160 | 4,831,812 | 23,222 | 46.550 | 0.9755 |
| n512 row24k rerank100 | 192 | 4,831,812 | 23,222 | 51.072 | 0.9840 |
| n512 row25k rerank50 | 160 | 5,029,652 | 24,160 | 43.643 | 0.9755 |
| n512 row25k rerank50 | 192 | 5,029,652 | 24,160 | 50.188 | 0.9840 |
| n512 row25k rerank100 | 160 | 5,029,652 | 24,160 | 45.586 | 0.9755 |
| n512 row25k rerank100 | 192 | 5,029,652 | 24,160 | 50.036 | 0.9840 |

## Interpretation

This is negative evidence for using rerank width as the main Task 79 closure lever. Rerank width changes retained-candidate processing but does not directly reduce the scored candidate surface, and the measured recall remains materially below the required floor.

## Validation

- `target/debug/ecaz bench suite audit --config reviews/task-79/005-rerank-budget-probe/suite-rabitq-rerank-budget-probe.json`
- `target/debug/ecaz bench suite run --dry-run --config reviews/task-79/005-rerank-budget-probe/suite-rabitq-rerank-budget-probe.json`
- `target/debug/ecaz bench suite run --config reviews/task-79/005-rerank-budget-probe/suite-rabitq-rerank-budget-probe.json`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/005-rerank-budget-probe/artifacts/suite-manifest.json`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/005-rerank-budget-probe/artifacts/suite-manifest.json`

Suite status: completed 6, failed 0, skipped 0.
