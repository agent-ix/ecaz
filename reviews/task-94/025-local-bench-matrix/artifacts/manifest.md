# Task 94 Packet 025 Artifact Manifest

- head SHA: `13ade4900` before this packet commit
- code checkpoint under measurement: `187be1af1` (`Batch IVF PqFastScan scratch scoring`)
- task bucket: `reviews/task-94/025-local-bench-matrix/`
- lane / fixture / storage: LUT lane / local PG18 / IVF PqFastScan rerank-off 10k, 25k, 100k plus DiskANN grouped-PQ forced 50k, 100k / `storage_format=pq_fastscan`
- host: local Intel x86_64 AVX2
- database: `postgres`
- socket / port: `/home/peter/.pgrx` / `28818`
- timestamp: 2026-06-09
- isolated one-index-per-table surfaces: yes for Task 94 IVF fixtures; DiskANN reused existing `task67_local_fullq_*_diskann` fixtures with `ec_diskann.prefilter_kind=grouped_pq` forced
- AWS / CI: not run in this packet by user direction

## Setup

### `create-local-ivf-pqfastscan-rerank-off-matrix.sql`

- command: `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file reviews/task-94/025-local-bench-matrix/artifacts/create-local-ivf-pqfastscan-rerank-off-matrix.sql --log-output reviews/task-94/025-local-bench-matrix/artifacts/create-local-ivf-pqfastscan-rerank-off-matrix.log`
- result: created 25k and 100k IVF `pq_fastscan` rerank-off fixtures with `nlists=64`, `nprobe=64`, `training_sample_rows=2000`, `pq_group_size=8`

### DiskANN grouped-PQ probe logs

- `probe-diskann-50k-latency-counters.log`: default local DiskANN latency path emitted no direct grouped-PQ block rows.
- `probe-diskann-50k-grouped-pq-latency-counters.log`: forcing `ec_diskann.prefilter_kind=grouped_pq` emitted direct DiskANN grouped-PQ rows, so the matrix includes forced grouped-PQ DiskANN latency cells.

## Suite

### `task94-local-pqfastscan-matrix-suite.json`

- local `ecaz bench suite` config with 14 steps:
  - IVF 10k, 25k, 100k recall with scratch batch off/on, `queries_limit=100`, `nprobe=32,64`
  - IVF 10k, 25k, 100k latency with scratch batch off/on, `iterations=500`, `nprobe=32,64`, direct block counters enabled
  - DiskANN 50k, 100k forced grouped-PQ latency, `iterations=120`, `list_size=64,128`, direct block counters enabled

### `suite-audit-cli.log`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/025-local-bench-matrix/artifacts/suite-audit-cli.log bench suite audit --config reviews/task-94/025-local-bench-matrix/artifacts/task94-local-pqfastscan-matrix-suite.json`
- key result: `[suite:task94-local-pqfastscan-matrix] audit passed: 14 steps`

### `suite-run-cli.log`, `suite-manifest.json`, `results.jsonl`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/025-local-bench-matrix/artifacts/suite-run-cli.log bench suite run --config reviews/task-94/025-local-bench-matrix/artifacts/task94-local-pqfastscan-matrix-suite.json --artifact-dir reviews/task-94/025-local-bench-matrix/artifacts --manifest-output reviews/task-94/025-local-bench-matrix/artifacts/suite-manifest.json --results-output reviews/task-94/025-local-bench-matrix/artifacts/results.jsonl`
- key result: suite completed 14 steps and wrote `results.jsonl` plus `suite-manifest.json`

### `suite-report-cli.log`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/025-local-bench-matrix/artifacts/suite-report-cli.log bench suite report --manifest reviews/task-94/025-local-bench-matrix/artifacts/suite-manifest.json`
- key result: completed 14, failed 0, skipped 0, missing artifacts 0, stale 0

## Key Results

### IVF recall parity

| Fixture | nprobe | batch off recall / ndcg | batch on recall / ndcg |
| --- | ---: | --- | --- |
| 10k | 32 | `0.4620` / `0.9036` | `0.4620` / `0.9036` |
| 10k | 64 | `0.4660` / `0.9051` | `0.4660` / `0.9051` |
| 25k | 32 | `0.4870` / `0.9276` | `0.4870` / `0.9276` |
| 25k | 64 | `0.4900` / `0.9283` | `0.4900` / `0.9283` |
| 100k | 32 | `0.6350` / `0.9679` | `0.6350` / `0.9679` |
| 100k | 64 | `0.6360` / `0.9679` | `0.6360` / `0.9679` |

### IVF latency

| Fixture | nprobe | batch off p50 / p95 / p99 | batch on p50 / p95 / p99 |
| --- | ---: | --- | --- |
| 10k | 32 | `2.87 ms` / `3.14 ms` / `3.25 ms` | `2.90 ms` / `3.21 ms` / `3.55 ms` |
| 10k | 64 | `4.58 ms` / `4.95 ms` / `5.35 ms` | `4.63 ms` / `4.96 ms` / `5.42 ms` |
| 25k | 32 | `5.54 ms` / `6.06 ms` / `6.50 ms` | `5.61 ms` / `6.18 ms` / `6.63 ms` |
| 25k | 64 | `10.0 ms` / `11.0 ms` / `15.4 ms` | `9.92 ms` / `10.8 ms` / `13.6 ms` |
| 100k | 32 | `18.0 ms` / `21.4 ms` / `27.5 ms` | `17.9 ms` / `22.9 ms` / `25.8 ms` |
| 100k | 64 | `34.8 ms` / `42.8 ms` / `49.6 ms` | `34.6 ms` / `41.7 ms` / `48.1 ms` |

### Direct `[block-kernel-counters]` rows

The suite latency parser emitted direct `block_kernel_counters` metrics in `results.jsonl`, not just `[task87-counters]` compatibility lines.

| Surface / fixture | Label | ISA | kernel_candidates | scalar_candidates |
| --- | --- | --- | ---: | ---: |
| IVF 10k | `nprobe=32` | `avx2` | 2401600 | 0 |
| IVF 10k | `nprobe=32` | `scalar` | 0 | 7455 |
| IVF 10k | `nprobe=64` | `avx2` | 4992000 | 0 |
| IVF 10k | `nprobe=64` | `scalar` | 0 | 8000 |
| IVF 25k | `nprobe=32` | `avx2` | 6667840 | 0 |
| IVF 25k | `nprobe=32` | `scalar` | 0 | 7330 |
| IVF 25k | `nprobe=64` | `avx2` | 12496000 | 0 |
| IVF 25k | `nprobe=64` | `scalar` | 0 | 4000 |
| IVF 100k | `nprobe=32` | `avx2` | 24142560 | 0 |
| IVF 100k | `nprobe=32` | `scalar` | 0 | 7530 |
| IVF 100k | `nprobe=64` | `avx2` | 50000000 | 0 |
| DiskANN 50k | `list_size=64` | `avx2` | 6432 | 0 |
| DiskANN 50k | `list_size=64` | `scalar` | 0 | 145026 |
| DiskANN 50k | `list_size=128` | `avx2` | 6592 | 0 |
| DiskANN 50k | `list_size=128` | `scalar` | 0 | 240260 |
| DiskANN 100k | `list_size=64` | `avx2` | 6464 | 0 |
| DiskANN 100k | `list_size=64` | `scalar` | 0 | 161329 |
| DiskANN 100k | `list_size=128` | `avx2` | 6688 | 0 |
| DiskANN 100k | `list_size=128` | `scalar` | 0 | 273395 |

### DiskANN forced grouped-PQ latency

| Fixture | list_size | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| DiskANN 50k | 64 | `22.3 ms` | `32.8 ms` | `38.3 ms` |
| DiskANN 50k | 128 | `18.7 ms` | `21.9 ms` | `23.7 ms` |
| DiskANN 100k | 64 | `43.9 ms` | `60.3 ms` | `74.5 ms` |
| DiskANN 100k | 128 | `34.3 ms` | `48.8 ms` | `66.7 ms` |

## Interpretation

- Local IVF PqFastScan batch-on recall is byte-identical at the benchmark level across 10k, 25k, and 100k.
- Local IVF production scans now emit AVX2 block rows and scalar-tail rows for PqFastScan through the scratch SoA path.
- End-to-end local IVF latency is mixed: small local regressions at 10k and 25k/nprobe32, small wins at 25k/nprobe64 and 100k/nprobe64, and p99 wins at the larger cells.
- Forced grouped-PQ DiskANN emits direct AVX2 rows, but most candidates remain scalar tails in the current local traversal shape; this packet proves attribution, not a DiskANN speedup claim.
- Graviton 4 SVE2 runtime dispatch/vector-length evidence and final AWS closeout remain deferred.
