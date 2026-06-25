---
head_sha: 25250bf58a3b3dd405d0d3f35015740095dd271d
task: task-119
packet: reviews/task-119/005-sidecar-rerank-m5-counter-matrix
host_class: m5-local
date: 2026-06-24
---

# Task 119 M5 Counter-Bearing Sidecar Matrix Manifest

## Scope

This packet reruns the Task 119 required sidecar matrix after
`4614d4c0ef8dbf4b8072aaa60773325f4a74b7f5` added explicit sidecar-rerank
counter columns.

Measured matrix:

```text
HNSW RaBitQ 1-bit candidate frontier + second-stage rerank representation
```

Variants:

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

All runs used `read_mode=free`, so sidecar/source I/O is intentionally excluded
and `heap_source_reads_*` is `0`. This packet supersedes
`reviews/task-119/003-sidecar-rerank-m5-matrix/` for free-I/O matrix rows
because it includes explicit counter columns.

The 1536-dimensional `turboquant_4bit` lane is the special tiled no-QJL lane:
4 MSE bits/dim, 0 QJL bits/dim, and 16 MSE centroids. The other TurboQuant
lanes use the QJL-active composition from Task 119.

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
./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json --only sidecar-10k-hnsw-rabitq-required-rerank-matrix --artifact-dir reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts --manifest-output reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-manifest.10k.json --results-output reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-results.10k.jsonl --log-file reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-run.10k.log

./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json --only sidecar-50k-hnsw-rabitq-required-rerank-matrix --artifact-dir reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts --manifest-output reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-manifest.50k.json --results-output reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-results.50k.jsonl --log-file reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-run.50k.log

./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-sidecar-rerank-matrix.json --only sidecar-100k-hnsw-rabitq-required-rerank-matrix --artifact-dir reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts --manifest-output reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-manifest.100k.json --results-output reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-results.100k.jsonl --log-file reviews/task-119/005-sidecar-rerank-m5-counter-matrix/artifacts/suite-run.100k.log
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
| `sidecar-10k-hnsw-rabitq-required-rerank-matrix.log` | table log | Full p50/p95/p99 table with counters |
| `sidecar-50k-hnsw-rabitq-required-rerank-matrix.log` | table log | Full p50/p95/p99 table with counters |
| `sidecar-100k-hnsw-rabitq-required-rerank-matrix.log` | table log | Full p50/p95/p99 table with counters |

## Counter Semantics

At `ef_search=1000`, every variant reports:

- `frontier_p50=1000`
- `reranked_p50=1000`
- `emitted_p50=10`
- `heap_source_reads_p50=0`

The zero read count is expected for `read_mode=free`; it documents that this is
the CPU/scoring upper bound and not a production heap/source read measurement.

## Key Results at `ef_search=1000`

| Scale | Variant | Recall@10 | frontier p50 | reranked p50 | heap/source reads p50 | emitted p50 | total bound p50 | bytes/vector | sidecar size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | `f32` | 0.9765 | 1000 | 1000 | 0 | 10 | 39.315 ms | 6144 | 58.59 MiB |
| 10k | `rabitq8` | 0.9650 | 1000 | 1000 | 0 | 10 | 25.888 ms | 1548 | 14.76 MiB |
| 10k | `turboquant_4bit` | 0.9535 | 1000 | 1000 | 0 | 10 | 26.171 ms | 772 | 7.36 MiB |
| 10k | `turboquant_8bit` | 0.9730 | 1000 | 1000 | 0 | 10 | 98.752 ms | 1540 | 14.69 MiB |
| 50k | `f32` | 0.9885 | 1000 | 1000 | 0 | 10 | 39.334 ms | 6144 | 292.97 MiB |
| 50k | `rabitq8` | 0.9475 | 1000 | 1000 | 0 | 10 | 26.326 ms | 1548 | 73.81 MiB |
| 50k | `turboquant_4bit` | 0.9390 | 1000 | 1000 | 0 | 10 | 26.863 ms | 772 | 36.81 MiB |
| 50k | `turboquant_8bit` | 0.9790 | 1000 | 1000 | 0 | 10 | 102.636 ms | 1540 | 73.43 MiB |
| 100k | `f32` | 0.9850 | 1000 | 1000 | 0 | 10 | 47.411 ms | 6144 | 585.94 MiB |
| 100k | `rabitq8` | 0.9420 | 1000 | 1000 | 0 | 10 | 34.405 ms | 1548 | 147.63 MiB |
| 100k | `turboquant_4bit` | 0.9415 | 1000 | 1000 | 0 | 10 | 35.115 ms | 772 | 73.62 MiB |
| 100k | `turboquant_8bit` | 0.9760 | 1000 | 1000 | 0 | 10 | 112.756 ms | 1540 | 146.87 MiB |

The JSONL artifacts contain all variants and all `ef_search` values, including
p50/p95/p99 latency fields and the new counter fields.

## Interpretation

- The free-I/O matrix now explicitly proves per-representation frontier,
  reranked, emitted, and read-count behavior at 10k/50k/100k.
- `f32` remains the recall ceiling but is too large to be the storage-saving
  answer by itself.
- `turboquant_4bit` remains the best compact CPU/scoring Pareto lane in this
  harness, with near-`rabitq8` latency and about half the bytes/vector.
- `turboquant_8bit` approaches `f32` recall but is much slower in this scoring
  implementation.

## Remaining Gap

This packet still does not close Task 119 by itself because it intentionally
uses `read_mode=free`. A final production-style sidecar read packet is still
needed if the closeout requires nonzero heap/source read latency and read
counts on the M5 host.
