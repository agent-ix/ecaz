# Review Request: Leaf Prefix Row Cap Probe

## Scope

Task 79 RaBitQ-only negative-evidence packet for a fixed leaf-prefix row cap experiment. This packet checks whether scanning only a fixed prefix of each selected leaf can directly reduce the scored candidate surface enough to satisfy Task 79.

## Result

The fixed-prefix row cap directly reduces candidates, but it collapses recall. Candidate counts fall to 3.3M to 4.0M in the best rows, and several p50 values are under 45 ms, but recall ranges only from 0.6730 to 0.8175.

| step | nprobe | candidates | routes | p50 ms | recall |
| --- | ---: | ---: | ---: | ---: | ---: |
| n512 row26k leaf160 | 192 | 3,311,427 | 25,116 | 43.185 | 0.6730 |
| n512 row26k leaf160 | 256 | 3,311,427 | 25,116 | 50.545 | 0.6785 |
| n512 row26k leaf192 | 192 | 3,719,442 | 25,116 | 45.468 | 0.7315 |
| n512 row26k leaf192 | 256 | 3,719,442 | 25,116 | 53.225 | 0.7385 |
| n512 row30k leaf160 | 192 | 3,813,970 | 28,958 | 42.982 | 0.6730 |
| n512 row30k leaf160 | 256 | 3,813,970 | 28,958 | 50.081 | 0.6785 |
| n256 row26k leaf320 | 128 | 3,537,649 | 13,270 | 39.951 | 0.7570 |
| n256 row26k leaf320 | 192 | 3,537,649 | 13,270 | 53.726 | 0.7610 |
| n256 row26k leaf320 | 256 | 3,537,649 | 13,270 | 65.610 | 0.7620 |
| n256 row26k leaf384 | 128 | 3,981,136 | 13,270 | 40.651 | 0.8115 |
| n256 row26k leaf384 | 192 | 3,981,136 | 13,270 | 55.313 | 0.8165 |
| n256 row26k leaf384 | 256 | 3,981,136 | 13,270 | 69.934 | 0.8175 |

## Interpretation

This is useful evidence but not a viable implementation path. It proves that reducing the scored candidate surface can move latency, but a query-oblivious prefix is the wrong selector. Task 79 needs query-aware subleaf pruning rather than build-time row ordering or fixed per-leaf prefix caps.

## Validation

- `cargo build -p ecaz-cli`
- `target/debug/ecaz dev install ecaz-pg-test --pg 18`
- `target/debug/ecaz bench suite audit --config reviews/task-79/006-leaf-prefix-row-cap/suite-rabitq-leaf-prefix-row-cap.json`
- `target/debug/ecaz bench suite run --dry-run --config reviews/task-79/006-leaf-prefix-row-cap/suite-rabitq-leaf-prefix-row-cap.json`
- `target/debug/ecaz bench suite run --config reviews/task-79/006-leaf-prefix-row-cap/suite-rabitq-leaf-prefix-row-cap.json`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/006-leaf-prefix-row-cap/artifacts/suite-manifest.json`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/006-leaf-prefix-row-cap/artifacts/suite-manifest.json`

Suite status: completed 8, failed 0, skipped 0.
