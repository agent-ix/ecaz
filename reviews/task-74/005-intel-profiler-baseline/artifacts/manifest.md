# Task 74 Intel Profiler Baseline Manifest

- Head SHA at packet creation: `ab501c2e9`
- Task bucket: `reviews/task-74/005-intel-profiler-baseline/`
- Lane: `intel-local`
- Fixture: `ec_real_100k`
- Surfaces:
  - SPIRE high-recall: `task74_intel_spire_highrecall_tg128_b0`,
    `top_graph_search_list_size=128`, `boundary_replica_count=0`,
    `nprobe=96`, `rerank_width=25`, `storage_format=turboquant`
  - IVF control: `task74_intel_ivf_control`, `nprobe=96`,
    `rerank_width=500`, `storage_format=pq_fastscan`, `pq_group_size=8`
- Storage layout: isolated one-index-per-table surfaces
- Runner: `ecaz bench suite`
- Suite config:
  `reviews/task-74/005-intel-profiler-baseline/artifacts/suite.json`
- Timestamp: `2026-05-31T16:26:53Z`

## Commands

Suite audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-74/005-intel-profiler-baseline/artifacts/suite.json --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-74/005-intel-profiler-baseline/artifacts/suite-audit.log
```

Suite run:

```text
target/debug/ecaz bench suite run --config reviews/task-74/005-intel-profiler-baseline/artifacts/suite.json --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-74/005-intel-profiler-baseline/artifacts/suite-run.log
```

SPIRE profiler driver:

```text
target/debug/ecaz bench latency --database postgres --host /home/peter/.pgrx --port 28818 --prefix task74_intel_spire_highrecall_tg128_b0 --profile ec_spire --k 10 --sweep 96 --rerank-width 25 --iterations 2000 --concurrency 1 --force-index --log-output reviews/task-74/005-intel-profiler-baseline/artifacts/profile-driver-spire-nprobe96.log
```

SPIRE perf capture:

```text
perf record -F 99 -e cpu-clock -g -p 2513396 -o reviews/task-74/005-intel-profiler-baseline/artifacts/intel-spire-nprobe96.perf.data -- sleep 30
```

IVF profiler driver:

```text
target/debug/ecaz bench latency --database postgres --host /home/peter/.pgrx --port 28818 --prefix task74_intel_ivf_control --profile ec_ivf --k 10 --sweep 96 --rerank-width 500 --iterations 2000 --concurrency 1 --force-index --log-output reviews/task-74/005-intel-profiler-baseline/artifacts/profile-driver-ivf-nprobe96.log
```

IVF perf capture:

```text
perf record -F 99 -e cpu-clock -g -p 2514409 -o reviews/task-74/005-intel-profiler-baseline/artifacts/intel-ivf-nprobe96.perf.data -- sleep 30
```

Flamegraph rendering:

```text
/home/peter/.cargo/bin/flamegraph --perfdata reviews/task-74/005-intel-profiler-baseline/artifacts/intel-spire-nprobe96.perf.data --output reviews/task-74/005-intel-profiler-baseline/artifacts/intel-spire-nprobe96-flamegraph.svg --title task74-intel-spire-nprobe96 --deterministic
/home/peter/.cargo/bin/flamegraph --perfdata reviews/task-74/005-intel-profiler-baseline/artifacts/intel-ivf-nprobe96.perf.data --output reviews/task-74/005-intel-profiler-baseline/artifacts/intel-ivf-nprobe96-flamegraph.svg --title task74-intel-ivf-nprobe96 --deterministic
```

## Key Results

- Suite status: completed `6`, failed `0`, skipped `0`, missing artifacts `0`.
- SPIRE suite latency at nprobe `96`: p50 `137.9 ms`, p95 `151.4 ms`,
  p99 `161.8 ms`, mean `138.8 ms`, count `200`.
- IVF suite latency at nprobe `96`: p50 `37.8 ms`, p95 `44.9 ms`,
  p99 `49.8 ms`, mean `38.6 ms`, count `200`.
- SPIRE profiler driver: p50 `133.7 ms`, p95 `147.2 ms`, p99 `156.0 ms`,
  count `2000`.
- IVF profiler driver: p50 `36.9 ms`, p95 `41.5 ms`, p99 `49.3 ms`,
  count `2000`.
- SPIRE perf sample: `2,947` samples, `0` lost samples.
- IVF perf sample: `2,894` samples, `0` lost samples.
- SPIRE top symbol: `75.06%` self-time in
  `ecaz::quant::prod::ProdQuantizer::score_ip_from_split_parts`.
- IVF top symbols: `42.92%` self-time in
  `ecaz::am::ec_ivf::quantizer::IvfQuantizer::score_ip_from_parts_with_min_bound`
  and `30.96%` self-time in `randomize_mem`.
- Identifiable SPIRE-specific non-scoring orchestration in the sample:
  approximately `4.9%`.

## Limitations

- This is Intel-local Linux perf evidence, not M5 `samply` evidence.
- System-wide `perf -a` was blocked by `perf_event_paranoid=2`; the successful
  evidence attaches to individual PostgreSQL backend PIDs.
- The callback-boundary split requested by the reviewer is partially visible
  through Rust SPIRE symbols, but not precise enough to claim a full
  `pg_am_callback!` boundary decomposition.
