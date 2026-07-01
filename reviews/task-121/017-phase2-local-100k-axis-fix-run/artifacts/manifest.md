# Task 121 Packet 017 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/017-phase2-local-100k-axis-fix-run/`
- Head SHA: `4ffd985155bbf686bae51abc69de022fb1653836`
- Suite: `task121-phase2-local-100k-axis-fix-run`
- Suite manifest generated: `2026-06-24T06:17:05Z`
- Packet manifest written: `2026-06-25T06:57:31Z`
- Lane: local PG18, single PostgreSQL instance
- Database: `tqvector_bench_task121`
- Host/port: `/tmp:28818`
- Backend artifact: `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- Backend SHA256: `f35657b0d65ecd87ab80db780efbed51d5d0acc4234a099f61bb02b079ab9cd2`
- Fixture: `data/staged-current/ec_real_100k_corpus.tsv`
- Fixture SHA from loader logs: `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`
- Query SHA from loader logs: `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Storage format: `rabitq`
- Rerank mode: default
- Isolation: one table/index per cell; no shared-table surface

## Command

```text
target/debug/ecaz --database tqvector_bench_task121 --host /tmp --port 28818 bench suite run --config reviews/task-121/017-phase2-local-100k-axis-fix-run/artifacts/suite-phase2-local-100k-axis-fix-run.json --manifest-output reviews/task-121/017-phase2-local-100k-axis-fix-run/artifacts/suite-phase2-local-100k-axis-fix-run-manifest.json --results-output reviews/task-121/017-phase2-local-100k-axis-fix-run/artifacts/suite-phase2-local-100k-axis-fix-run-results.jsonl --log-file reviews/task-121/017-phase2-local-100k-axis-fix-run/artifacts/suite-phase2-local-100k-axis-fix-run-fresh.log
```

## Status

```text
steps=50
succeeded=50
results_jsonl_lines=1921
```

All 16 pipeline cells completed with:

```text
funnel_rows=1800
stage_containment_rows=10800
```

The streamed `pipeline-100k_*-funnel.jsonl`,
`pipeline-100k_*-stage-containment.jsonl`, and
`truth-cache-100k-q200-k10.json` files are local-only diagnostics and are not
commit artifacts.

## Artifacts

- `summary-100k-axis-fix-run.md`: compact storage and recall/latency summary.
- `suite-phase2-local-100k-axis-fix-run.json`: SuiteConfig used for the run.
- `suite-phase2-local-100k-axis-fix-run-audit.log`: audit command output.
- `suite-phase2-local-100k-axis-fix-run-fresh.log`: suite execution log.
- `suite-phase2-local-100k-axis-fix-run-manifest.json`: final suite manifest.
- `suite-phase2-local-100k-axis-fix-run-results.jsonl`: structured suite
  result rows.
- `precheck-host.log`: host precheck.
- `load-100k_*.log`: per-cell load/build logs.
- `storage-100k_*.log`: per-cell storage logs.
- `pipeline-100k_*.log`: per-cell compact pipeline logs.
- `truth-cache-100k-q200-k10.log`: truth-cache build log.

## Key Result Lines

Selected recall/latency:

```text
b0_tr10_f8:  r@8=0.7250 r@32=0.9310 r@96=0.9975 p50@32=1080.340 ms p50@96=3332.217 ms
b1_tr50_f16: r@8=0.8645 r@32=0.9810 r@96=0.9995 p50@32=1521.840 ms p50@96=3710.127 ms
b2_tr50_f16: r@8=0.9035 r@32=0.9850 r@96=1.0000 p50@32=1942.887 ms p50@96=4197.764 ms
b4_tr50_f8:  r@8=0.9330 r@32=0.9895 r@96=1.0000 p50@32=2544.873 ms p50@96=4498.213 ms
b4_tr50_f16: r@8=0.9340 r@32=0.9915 r@96=1.0000 p50@32=2589.751 ms p50@96=4504.694 ms
```

Storage:

```text
b0 family: ~79.6-79.8 MiB index, ~835-837 B/index-row, 1.6 GiB total
b1 family: ~157.8-157.9 MiB index, ~1655-1656 B/index-row, 1.7 GiB total
b2 family: ~235.9-236.0 MiB index, ~2474-2475 B/index-row, 1.8 GiB total
b4 family: ~392.2-392.3 MiB index, ~4112-4113 B/index-row, 1.9 GiB total
```
