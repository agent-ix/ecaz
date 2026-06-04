# Review Request: Row-Budget Geometry Refinement

## Scope

Task 79 RaBitQ-only no-format-change measurement over the untested n384 and n448 route-time row-budget bracket. This packet checks whether existing SPIRE route geometry can satisfy the Task 79 candidate, recall, and latency gates without introducing leaf-local subleaf pruning.

## Result

No row satisfies all Task 79 gates. The n384 rows can pass candidate and recall at nprobe224, but p50 stays around 57 ms. The n448 rows improve lower-nprobe latency, but those rows miss recall; the recall-passing n448 rows remain 58.515 ms to 62.649 ms p50.

| nlists | row budget | nprobe | candidates | routes | p50 ms | recall |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 384 | 24k | 192 | 4,836,456 | 18,790 | 53.343 | 0.9910 |
| 384 | 24k | 224 | 4,836,456 | 18,790 | 57.000 | 0.9955 |
| 384 | 24k | 256 | 4,836,456 | 18,790 | 63.022 | 0.9975 |
| 384 | 25k | 192 | 5,036,739 | 19,531 | 50.814 | 0.9910 |
| 384 | 25k | 224 | 5,036,739 | 19,531 | 57.164 | 0.9955 |
| 384 | 25k | 256 | 5,036,739 | 19,531 | 63.290 | 0.9975 |
| 384 | 26k | 192 | 5,236,559 | 20,282 | 50.802 | 0.9910 |
| 384 | 26k | 224 | 5,236,559 | 20,282 | 56.739 | 0.9955 |
| 384 | 26k | 256 | 5,236,559 | 20,282 | 62.737 | 0.9975 |
| 448 | 24k | 192 | 4,833,877 | 20,972 | 48.898 | 0.9870 |
| 448 | 24k | 224 | 4,833,877 | 20,972 | 54.662 | 0.9890 |
| 448 | 24k | 256 | 4,833,877 | 20,972 | 62.649 | 0.9935 |
| 448 | 25k | 192 | 5,036,640 | 21,820 | 48.958 | 0.9870 |
| 448 | 25k | 224 | 5,036,640 | 21,820 | 53.674 | 0.9890 |
| 448 | 25k | 256 | 5,036,640 | 21,820 | 58.515 | 0.9935 |
| 448 | 26k | 192 | 5,233,192 | 22,611 | 49.711 | 0.9870 |
| 448 | 26k | 224 | 5,233,192 | 22,611 | 57.483 | 0.9890 |
| 448 | 26k | 256 | 5,233,192 | 22,611 | 60.388 | 0.9935 |

## Interpretation

This closes the no-format-change route-geometry bracket. Existing route-time row budgets can bring scored candidates near the Task 79 candidate gate, but the remaining latency/recall tradeoff is still whole-leaf granularity. The next implementation slice should follow ADR-074: query-aware leaf-local block summaries reachable before row-segment reads, with a V3 reject-unknown format and full-leaf fallback for diagnostics.

## Validation

- `cargo build -p ecaz-cli`
- `target/debug/ecaz dev install ecaz-pg-test --pg 18`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/008-row-budget-geometry-refinement/artifacts/pg18-restart.log restart -m fast`
- `jq empty reviews/task-79/008-row-budget-geometry-refinement/suite-rabitq-row-budget-geometry-refinement.json`
- `target/debug/ecaz bench suite audit --config reviews/task-79/008-row-budget-geometry-refinement/suite-rabitq-row-budget-geometry-refinement.json`
- `target/debug/ecaz bench suite run --dry-run --config reviews/task-79/008-row-budget-geometry-refinement/suite-rabitq-row-budget-geometry-refinement.json`
- `target/debug/ecaz bench suite run --config reviews/task-79/008-row-budget-geometry-refinement/suite-rabitq-row-budget-geometry-refinement.json`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/008-row-budget-geometry-refinement/artifacts/suite-manifest.json`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/008-row-budget-geometry-refinement/artifacts/suite-manifest.json`

Suite status: completed 9, failed 0, skipped 0.
