---
head_sha: 7677f047a0005a2549192f62efffaf74fdf705ae
task: task-119
packet: reviews/task-119/003-sidecar-rerank-m5-matrix
host_class: m5-local
date: 2026-06-24
---

# Task 119 M5 Sidecar Rerank Matrix Manifest

## Scope

This packet measures the Task 119 required matrix:

```text
HNSW RaBitQ 1-bit candidate frontier + second-stage rerank representation
```

The measured rerank representations are:

- `f32`
- `rabitq2`
- `rabitq4`
- `rabitq8`
- `turboquant_2bit`
- `turboquant_3bit`
- `turboquant_4bit`
- `turboquant_5bit`
- `turboquant_6bit`
- `turboquant_7bit`
- `turboquant_8bit`

All runs used `read_mode=free`, so sidecar I/O is intentionally excluded.
These are CPU/scoring upper-bound measurements over a retained candidate
frontier, not production heap/sidecar read measurements.

The current 1536-dimensional `turboquant_4bit` lane is the special tiled
no-QJL lane: 4 MSE bits/dim, 0 QJL bits/dim, and 16 MSE centroids. The other
TurboQuant lanes use the QJL-active composition documented in Task 119.

## Suite Config

- Config: `crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json`
- Runner: `ecaz bench suite`
- Database: `tqvector_task119_m5_release2`
- PG socket: `/Users/peter/.pgrx`
- PG port: `28818`
- Profile: `ec_hnsw`
- Candidate frontier: existing HNSW RaBitQ index, one isolated index per corpus table
- `candidate_k`: `1000`
- `k`: `10`
- `queries_limit`: `200`
- `ef_search` sweep: `320`, `500`, `1000`
- `read_modes`: `free`
- `allow_unsafe_index_shape`: `true`

## Commands

```sh
./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json --only sidecar-10k-hnsw-rabitq-required-rerank-matrix --artifact-dir reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts --manifest-output reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-manifest.10k.json --results-output reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-results.10k.jsonl --log-file reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-run.10k.log

./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json --only sidecar-50k-hnsw-rabitq-required-rerank-matrix --artifact-dir reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts --manifest-output reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-manifest.50k.json --results-output reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-results.50k.jsonl --log-file reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-run.50k.log

./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json --only sidecar-100k-hnsw-rabitq-required-rerank-matrix --artifact-dir reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts --manifest-output reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-manifest.100k.json --results-output reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-results.100k.jsonl --log-file reviews/task-119/003-sidecar-rerank-m5-matrix/artifacts/suite-run.100k.log
```

## Artifacts

| Artifact | Rows / content | Notes |
| --- | ---: | --- |
| `suite-results.10k.jsonl` | 33 | 11 variants x 3 `ef_search` values |
| `suite-results.50k.jsonl` | 33 | 11 variants x 3 `ef_search` values |
| `suite-results.100k.jsonl` | 33 | 11 variants x 3 `ef_search` values |
| `suite-manifest.10k.json` | 3 steps | 10k step succeeded; others skipped |
| `suite-manifest.50k.json` | 3 steps | 50k step succeeded; others skipped |
| `suite-manifest.100k.json` | 3 steps | 100k step succeeded; others skipped |
| `sidecar-10k-hnsw-rabitq-required-rerank-matrix.log` | table log | Full p50/p95/p99 table |
| `sidecar-50k-hnsw-rabitq-required-rerank-matrix.log` | table log | Full p50/p95/p99 table |
| `sidecar-100k-hnsw-rabitq-required-rerank-matrix.log` | table log | Full p50/p95/p99 table |
| `suite-run.10k.log` | command log | Suite launch/result writes |
| `suite-run.50k.log` | command log | Suite launch/result writes |
| `suite-run.100k.log` | command log | Suite launch/result writes |

## Key Results at `ef_search=1000`

| Scale | Variant | Recall@10 | NDCG@10 | candidate SQL p50 | sidecar score p50 | total bound p50 | bytes/vector | sidecar size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | `f32` | 0.9765 | 0.9956 | 14.616 ms | 22.342 ms | 36.953 ms | 6144 | 58.59 MiB |
| 10k | `rabitq2` | 0.9255 | 0.9948 | 14.616 ms | 17.168 ms | 31.813 ms | 396 | 3.78 MiB |
| 10k | `rabitq4` | 0.9580 | 0.9955 | 14.616 ms | 14.254 ms | 28.932 ms | 780 | 7.44 MiB |
| 10k | `rabitq8` | 0.9650 | 0.9955 | 14.616 ms | 9.542 ms | 24.147 ms | 1548 | 14.76 MiB |
| 10k | `turboquant_2bit` | 0.8820 | 0.9930 | 14.616 ms | 75.029 ms | 89.674 ms | 388 | 3.70 MiB |
| 10k | `turboquant_3bit` | 0.9090 | 0.9943 | 14.616 ms | 75.657 ms | 90.293 ms | 580 | 5.53 MiB |
| 10k | `turboquant_4bit` | 0.9535 | 0.9953 | 14.616 ms | 10.185 ms | 24.798 ms | 772 | 7.36 MiB |
| 10k | `turboquant_5bit` | 0.9540 | 0.9954 | 14.616 ms | 72.355 ms | 87.001 ms | 964 | 9.19 MiB |
| 10k | `turboquant_6bit` | 0.9635 | 0.9955 | 14.616 ms | 81.130 ms | 95.753 ms | 1156 | 11.02 MiB |
| 10k | `turboquant_7bit` | 0.9705 | 0.9956 | 14.616 ms | 79.220 ms | 93.889 ms | 1348 | 12.86 MiB |
| 10k | `turboquant_8bit` | 0.9730 | 0.9956 | 14.616 ms | 83.705 ms | 98.403 ms | 1540 | 14.69 MiB |
| 50k | `f32` | 0.9885 | 0.9993 | 17.236 ms | 22.451 ms | 39.795 ms | 6144 | 292.97 MiB |
| 50k | `rabitq2` | 0.8815 | 0.9969 | 17.236 ms | 17.119 ms | 34.344 ms | 396 | 18.88 MiB |
| 50k | `rabitq4` | 0.9390 | 0.9988 | 17.236 ms | 14.209 ms | 31.575 ms | 780 | 37.19 MiB |
| 50k | `rabitq8` | 0.9475 | 0.9989 | 17.236 ms | 9.620 ms | 26.922 ms | 1548 | 73.81 MiB |
| 50k | `turboquant_2bit` | 0.7805 | 0.9902 | 17.236 ms | 75.053 ms | 92.555 ms | 388 | 18.50 MiB |
| 50k | `turboquant_3bit` | 0.8665 | 0.9960 | 17.236 ms | 75.735 ms | 93.009 ms | 580 | 27.66 MiB |
| 50k | `turboquant_4bit` | 0.9390 | 0.9989 | 17.236 ms | 10.340 ms | 27.593 ms | 772 | 36.81 MiB |
| 50k | `turboquant_5bit` | 0.9415 | 0.9989 | 17.236 ms | 71.472 ms | 88.650 ms | 964 | 45.97 MiB |
| 50k | `turboquant_6bit` | 0.9640 | 0.9992 | 17.236 ms | 81.022 ms | 98.411 ms | 1156 | 55.12 MiB |
| 50k | `turboquant_7bit` | 0.9740 | 0.9992 | 17.236 ms | 78.460 ms | 95.755 ms | 1348 | 64.28 MiB |
| 50k | `turboquant_8bit` | 0.9790 | 0.9992 | 17.236 ms | 85.923 ms | 103.297 ms | 1540 | 73.43 MiB |
| 100k | `f32` | 0.9850 | 0.9993 | 26.498 ms | 22.609 ms | 49.171 ms | 6144 | 585.94 MiB |
| 100k | `rabitq2` | 0.8760 | 0.9971 | 26.498 ms | 17.698 ms | 44.512 ms | 396 | 37.77 MiB |
| 100k | `rabitq4` | 0.9320 | 0.9989 | 26.498 ms | 14.653 ms | 41.525 ms | 780 | 74.39 MiB |
| 100k | `rabitq8` | 0.9420 | 0.9990 | 26.498 ms | 9.635 ms | 36.134 ms | 1548 | 147.63 MiB |
| 100k | `turboquant_2bit` | 0.7895 | 0.9912 | 26.498 ms | 75.216 ms | 101.796 ms | 388 | 37.00 MiB |
| 100k | `turboquant_3bit` | 0.8640 | 0.9963 | 26.498 ms | 74.838 ms | 101.569 ms | 580 | 55.31 MiB |
| 100k | `turboquant_4bit` | 0.9415 | 0.9990 | 26.498 ms | 10.053 ms | 36.586 ms | 772 | 73.62 MiB |
| 100k | `turboquant_5bit` | 0.9425 | 0.9990 | 26.498 ms | 69.989 ms | 96.679 ms | 964 | 91.93 MiB |
| 100k | `turboquant_6bit` | 0.9555 | 0.9992 | 26.498 ms | 80.659 ms | 107.495 ms | 1156 | 110.24 MiB |
| 100k | `turboquant_7bit` | 0.9715 | 0.9993 | 26.498 ms | 80.150 ms | 106.755 ms | 1348 | 128.56 MiB |
| 100k | `turboquant_8bit` | 0.9760 | 0.9993 | 26.498 ms | 84.678 ms | 111.113 ms | 1540 | 146.87 MiB |

## Interpretation

- `f32` remains the highest-recall rerank representation, but it costs 6144
  bytes/vector and 585.94 MiB of sidecar storage at 100k.
- `rabitq8` is the fastest scoring lane at every scale, but recall trails
  `f32` by 4.3 points at 100k `ef_search=1000`.
- `turboquant_4bit` is the strongest compact Pareto lane in this harness:
  at 100k it nearly matches `rabitq8` recall and latency while using about
  half the sidecar bytes/vector. This is the 1536-dimensional no-QJL special
  lane, not the general QJL-active 3 MSE + 1 QJL composition.
- QJL-active TurboQuant 5/6/7/8 improve recall as bits increase, but their
  score latency is much higher in this implementation. At 100k, `turboquant_8bit`
  reaches 0.9760 recall but has 111.113 ms total-bound p50.
- `turboquant_2bit` and `turboquant_3bit` are not viable in this matrix:
  they are slower than the compact RaBitQ lanes while losing more recall.

## Closeout Status

This packet satisfies the required 10k/50k/100k rerank-representation matrix for
M5 upper-bound sidecar scoring. It does not by itself close Task 119 because
the sidecar harness still lacks full task counters for visited graph nodes,
heap/source reads, build time, and production sidecar I/O. It reports candidate
frontier size, reranked candidate count, recall, NDCG, p50/p95/p99 latency, and
sidecar storage for every required representation.
