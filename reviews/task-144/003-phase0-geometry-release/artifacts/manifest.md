# Task 144 Packet 003 Artifact Manifest

- Head SHA: `ef3e9b4206d2e37c71dc53b13607a953ec173020`
- Branch: `task-144-spire-closure-ratio-pruning`
- Task bucket: `reviews/task-144/003-phase0-geometry-release`
- Captured: `2026-07-05T09:04:03-07:00`
- Lane: local PG18 scratch, release backend, single-node SPIRE geometry diagnostic
- Fixture: real corpus 50k and 100k, `nlists=1024`, `boundary_replica_count=0`, `storage_format=rabitq`, `queries_limit=200`, `k=10`
- Surface: isolated one-index-per-table prefixes `t144_50k_n1024_phase0` and `t144_100k_n1024_phase0`

## Commands

- `target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-144/003-phase0-geometry-release/artifacts/install-release-pg18.log`
- `target/release/ecaz bench suite audit --config reviews/task-144/003-phase0-geometry-release/artifacts/suite-task144-phase0-geometry-release.json --database tqvector_bench_task144 --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 --log-file reviews/task-144/003-phase0-geometry-release/artifacts/suite-audit-r3.log`
- `target/release/ecaz bench suite run --config reviews/task-144/003-phase0-geometry-release/artifacts/suite-task144-phase0-geometry-release.json --database tqvector_bench_task144 --host /home/peter/dev/ecaz/target/task144-pg18-socket --port 28818 --resume-from reviews/task-144/003-phase0-geometry-release/artifacts/suite-manifest.json --log-file reviews/task-144/003-phase0-geometry-release/artifacts/suite-run-r3.log`
- `script -q -c "cargo test -p ecaz-cli spire_pipeline --no-default-features" reviews/task-144/003-phase0-geometry-release/artifacts/cargo-test-ecaz-cli-spire-pipeline.log`

## Artifacts

- `suite-task144-phase0-geometry-release.json`: checked-in `ecaz bench suite` config.
- `suite-manifest.json`: final suite manifest; `completed=7 failed=0 skipped=0 stale=0`.
- `results.jsonl`: normalized suite rows.
- `geometry-50k-n1024.jsonl`: 50k leaf-size and true-neighbor concentration diagnostic rows.
- `geometry-100k-n1024.jsonl`: 100k leaf-size and true-neighbor concentration diagnostic rows.
- `geometry-50k-n1024.log`, `geometry-100k-n1024.log`: command logs for the geometry steps.
- `load-50k-n1024-index.log`, `load-100k-n1024-index.log`: cited load/build logs.
- `truth-cache-50k-q200-k10.log`, `truth-cache-100k-q200-k10.log`: exact-truth generation logs. The generated `truth-cache-*.json` files are intentionally uncommitted per repo policy.
- `install-release-pg18.log`: release backend install assertion, installed `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`, sha256 `160eaece45e684c920676067caf005c2159900b038e29eea3cf0eb9a791aad3f`.
- `precheck-host.log`: PG18 precheck row, `ecaz_build_profile = release`.
- `suite-audit-r3.log`, `suite-run-r3.log`: suite audit/run logs.
- `cargo-test-ecaz-cli-spire-pipeline.log`: focused validation log, `30 passed; 0 failed; 409 filtered out`.

## Key Result Lines

Leaf-size summaries:

| scale | leaves | rows | mean rows/leaf | p50 | p90 | p99 | max | CV | empty leaves |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | 1024 | 50000 | 48.828 | 45 | 85 | 130 | 196 | 0.550 | 0 |
| 100k | 1024 | 100000 | 97.656 | 89 | 161 | 246 | 347 | 0.496 | 0 |

True-neighbor list concentration across 200 q/k10 queries:

| scale | mode | epsilon | mean leaves/query | p50 | p90 | max | mean assignment rows/query | p90 assignment rows | missing truth |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | single assignment | n/a | 5.505 | 6 | 8 | 10 | 10.000 | 10 | 0 |
| 50k | simulated closure | 0.05 | 6.405 | 6 | 11 | 13 | 11.420 | 13 | 0 |
| 50k | simulated closure | 0.10 | 7.965 | 7 | 14 | 25 | 14.120 | 20 | 0 |
| 50k | simulated closure | 0.20 | 16.065 | 12 | 33 | 123 | 27.155 | 48 | 0 |
| 100k | single assignment | n/a | 5.615 | 6 | 8 | 10 | 10.000 | 10 | 0 |
| 100k | simulated closure | 0.05 | 6.635 | 7 | 10 | 13 | 11.460 | 14 | 0 |
| 100k | simulated closure | 0.10 | 8.935 | 9 | 15 | 28 | 14.625 | 20 | 0 |
| 100k | simulated closure | 0.20 | 18.840 | 17 | 36 | 96 | 28.755 | 48 | 0 |

Query metric sanity rows from `results.jsonl`:

| step | backend | latency_p50 | distinct_recall@k |
| --- | --- | ---: | ---: |
| `geometry-50k-n1024` | release | 178.927 ms | 0.9590 |
| `geometry-100k-n1024` | release | 371.564 ms | 0.9300 |

Interpretation: with current n1024 single assignment, true top-10 neighbors already span more than the desired 1-4 lists on average. The read-only IP-distance closure simulation increases row assignments/replication and does not reduce the unique leaf count needed to cover exact truth. Phase 1 should therefore be gated carefully: closure alone is unlikely to produce the 1-4 probe operating point without query-time ratio pruning and/or a separate balancing/multiprobe mechanism.
