---
head_sha: 323442f9007c4f2b859e305e7e407b8cd3dc3a71
task: task-119
packet: reviews/task-119/006-sidecar-rerank-m5-db-read
host_class: m5-local
date: 2026-06-24
---

# Task 119 M5 DB-Read Sidecar Rerank Manifest

## Scope

This packet measures production-style sidecar reads for the viable Task 119
rerank lanes identified by the full free-I/O matrix:

- `f32`
- `rabitq8`
- `turboquant_4bit`
- `turboquant_8bit`

The candidate frontier remains HNSW RaBitQ with `candidate_k=1000`. These runs
use `read_mode=tid-sorted`, so each query fetches sidecar payload rows through
PostgreSQL ordered by `ctid` before scoring.

This packet is not a replacement for the full required representation matrix in
`reviews/task-119/005-sidecar-rerank-m5-counter-matrix/`. It is a focused
production-I/O follow-up for the lanes that still looked viable after the full
CPU/scoring upper-bound matrix.

## Suite Config

- Config: `reviews/task-119/006-sidecar-rerank-m5-db-read/suite.json`
- Runner: `ecaz bench suite`
- Database: `tqvector_task119_m5_release2`
- PG socket: `/Users/peter/.pgrx`
- PG port: `28818`
- Profile: `ec_hnsw`
- `candidate_k`: `1000`
- `k`: `10`
- `queries_limit`: `200`
- `ef_search`: `1000`
- `read_modes`: `tid-sorted`
- `rebuild_sidecar_table`: `true`
- `allow_unsafe_index_shape`: `true`

## Validation

- `suite-audit.log`: suite audit passed.
- `suite-dry-run.log`: dry-run expanded all 3 DB-read steps.
- Result rows:
  - `suite-results.10k.jsonl`: 4 rows.
  - `suite-results.50k.jsonl`: 4 rows.
  - `suite-results.100k.jsonl`: 4 rows.

## Commands

```sh
./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite audit --config reviews/task-119/006-sidecar-rerank-m5-db-read/suite.json

./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-119/006-sidecar-rerank-m5-db-read/suite.json --artifact-dir reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts --manifest-output reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-manifest.dry-run.json

./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-119/006-sidecar-rerank-m5-db-read/suite.json --only sidecar-10k-hnsw-rabitq-db-read-viable-lanes --artifact-dir reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts --manifest-output reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-manifest.10k.json --results-output reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-results.10k.jsonl --log-file reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-run.10k.log

./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-119/006-sidecar-rerank-m5-db-read/suite.json --only sidecar-50k-hnsw-rabitq-db-read-viable-lanes --artifact-dir reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts --manifest-output reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-manifest.50k.json --results-output reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-results.50k.jsonl --log-file reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-run.50k.log

./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-119/006-sidecar-rerank-m5-db-read/suite.json --only sidecar-100k-hnsw-rabitq-db-read-viable-lanes --artifact-dir reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts --manifest-output reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-manifest.100k.json --results-output reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-results.100k.jsonl --log-file reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-run.100k.log
```

## Key Results

| Scale | Variant | Recall@10 | frontier p50 | heap/source reads p50 | sidecar I/O p50 | sidecar score p50 | total bound p50 | bytes/vector | sidecar size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | `f32` | 0.9765 | 1000 | 1000 | 23.917 ms | 38.131 ms | 77.489 ms | 6144 | 58.59 MiB |
| 10k | `rabitq8` | 0.9650 | 1000 | 1000 | 5.658 ms | 9.424 ms | 30.689 ms | 1548 | 14.76 MiB |
| 10k | `turboquant_4bit` | 0.9535 | 1000 | 1000 | 3.472 ms | 9.991 ms | 29.107 ms | 772 | 7.36 MiB |
| 10k | `turboquant_8bit` | 0.9730 | 1000 | 1000 | 6.144 ms | 82.553 ms | 104.535 ms | 1540 | 14.69 MiB |
| 50k | `f32` | 0.9885 | 1000 | 1000 | 23.158 ms | 38.488 ms | 80.183 ms | 6144 | 292.97 MiB |
| 50k | `rabitq8` | 0.9475 | 1000 | 1000 | 6.356 ms | 9.502 ms | 33.980 ms | 1548 | 73.81 MiB |
| 50k | `turboquant_4bit` | 0.9390 | 1000 | 1000 | 4.070 ms | 10.106 ms | 32.386 ms | 772 | 36.81 MiB |
| 50k | `turboquant_8bit` | 0.9790 | 1000 | 1000 | 6.826 ms | 86.588 ms | 111.725 ms | 1540 | 73.43 MiB |
| 100k | `f32` | 0.9850 | 1000 | 1000 | 23.285 ms | 37.768 ms | 86.750 ms | 6144 | 585.94 MiB |
| 100k | `rabitq8` | 0.9420 | 1000 | 1000 | 6.611 ms | 9.266 ms | 41.170 ms | 1548 | 147.63 MiB |
| 100k | `turboquant_4bit` | 0.9415 | 1000 | 1000 | 4.598 ms | 10.004 ms | 39.873 ms | 772 | 73.62 MiB |
| 100k | `turboquant_8bit` | 0.9760 | 1000 | 1000 | 8.245 ms | 84.082 ms | 117.642 ms | 1540 | 146.87 MiB |

## Interpretation

- Production-style `tid-sorted` reads do not change recall; they expose the
  sidecar fetch cost.
- `f32` remains the recall ceiling but needs 1000 heap/source-style payload
  reads per query and 6144 bytes/vector. At 100k, total-bound p50 is
  86.750 ms.
- `turboquant_4bit` remains the best compact practical lane in this focused
  DB-read packet: at 100k it reads 1000 sidecar rows, uses 772 bytes/vector,
  and has 39.873 ms total-bound p50.
- `rabitq8` is slightly faster than `turboquant_4bit` at 100k, but recall is
  nearly identical and storage is roughly 2x larger.
- `turboquant_8bit` preserves high recall but remains dominated by score
  latency, even when sidecar I/O is included.

## Closeout Impact

Together with packet `005`, this fills the M5 production-style read gap for the
viable lanes. It supports the Task 119 recommendation to keep the profile
experimental and iterate on `turboquant_4bit`/`rabitq8` style compact rerank,
not promote a production coarse-rerank profile yet.
