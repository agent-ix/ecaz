---
id: NFR-006
title: Async I/O Cold-Cache Performance
type: NFR
status: DRAFT
traces:
  - StR-004
  - FR-019
---
# NFR-006: Async I/O Cold-Cache Performance

## Statement

Ecaz SHALL deliver the cold-cache async-I/O performance improvements below on PostgreSQL 18 without regressing warm-cache performance.

### Cold-Cache HNSW Scan

On PG18 with `io_method=worker` or `io_method=io_uring` and `effective_io_concurrency=16`:
- Cold-cache HNSW top-10 query latency (50K × 1536-dim, 4-bit, m=8, ef_search=40) SHALL improve by ≥ 2x compared to `effective_io_concurrency=0` on the same dataset and hardware
- Cold-cache latency SHALL be reported for `io_method=sync`, `io_method=worker`, and `io_method=io_uring` (where available)

### Cold-Cache Linear Scan

On PG18 with sequential streaming reads:
- Cold-cache linear scan throughput SHALL be reported with and without the sequential `ReadStream`
- The streaming path SHALL not regress warm-cache performance

### Benchmark Methodology

All async I/O benchmarks SHALL:
1. Evict index buffers via `pg_buffercache_evict_relation()` before each cold-cache measurement
2. Report `io_method`, `effective_io_concurrency`, and `io_combine_limit` settings
3. Compare against the PG17 synchronous baseline on the same hardware
4. Measure at `effective_io_concurrency` values of 0, 4, 8, 16, and 32

### Key GUC Matrix

| GUC | Values to Test | Purpose |
|---|---|---|
| `io_method` | sync, worker, io_uring | I/O backend comparison |
| `effective_io_concurrency` | 0, 4, 8, 16, 32 | Prefetch depth |
| `io_combine_limit` | 8, 16, 32 | Pages per I/O op |
| `maintenance_io_concurrency` | 0, 16 | Vacuum/build I/O depth |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Cold-cache HNSW top-10 latency improvement (eic=16 vs eic=0) | >= 2x | >= 2x | Cold-Cache Latency Benchmark |
| Cold-cache latency reported per `io_method` (sync/worker/io_uring) | Reported | Reported | Cold-Cache Latency Benchmark |
| Warm-cache regression from streaming path | None | 0% | Latency Benchmark |


Results SHALL be reported in `BENCHMARKS.md` following the methodology in NFR-001.

## Verification

All async I/O benchmarks evict index buffers via `pg_buffercache_evict_relation()` before each cold-cache measurement, sweep `effective_io_concurrency` over {0,4,8,16,32}, report `io_method`/`io_combine_limit`, compare against the PG17 synchronous baseline on the same hardware, and assert the cold-cache improvement and absence of warm-cache regression.

