# Task 124 Packet 022 Artifact Manifest

- head SHA before packet: `04b55485c892a393574b83da71a6fe7da5a76fca`
- task bucket: `reviews/task-124`
- packet path: `reviews/task-124/022-tq-nprobe-frontier-refine`
- lane: TQ speed diagnostic / nprobe refinement
- fixture: `ec_real_100k`
- access method: `ec_ivf`
- storage format: `coarse_rerank`
- TQ config: `rerank_placement=index`, `rerank_format=turboquant`,
  `rerank_width=75`, `rerank_group_width=50`,
  `stage2_final_rerank_width=15`
- run surface: shared-table reuse of packet `020` isolated prefix
  `task124_tq_bscore_w75_g50_100k`
- date: 2026-06-29
- timestamp: 2026-06-30T02:08:47Z

## Commands

```text
target/release/ecaz bench suite audit --config reviews/task-124/022-tq-nprobe-frontier-refine/artifacts/task124-tq-nprobe-refine-100k-suite.json
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-124/022-tq-nprobe-frontier-refine/artifacts/task124-tq-nprobe-refine-100k-suite.json --manifest-output reviews/task-124/022-tq-nprobe-frontier-refine/artifacts/suite-manifest.json --results-output reviews/task-124/022-tq-nprobe-frontier-refine/artifacts/results.jsonl
target/release/ecaz --log-file reviews/task-124/022-tq-nprobe-frontier-refine/artifacts/suite-status.log bench suite status --manifest reviews/task-124/022-tq-nprobe-frontier-refine/artifacts/suite-manifest.json
target/release/ecaz --log-file reviews/task-124/022-tq-nprobe-frontier-refine/artifacts/suite-report.log bench suite report --manifest reviews/task-124/022-tq-nprobe-frontier-refine/artifacts/suite-manifest.json --results-output reviews/task-124/022-tq-nprobe-frontier-refine/artifacts/report-results.jsonl
```

## Artifact Inventory

- `task124-tq-nprobe-refine-100k-suite.json`: suite config.
- `suite-manifest.json`: completed suite manifest.
- `results.jsonl`: parsed suite results.
- `report-results.jsonl`: parsed report output.
- `suite-status.log`: 3-step suite status.
- `suite-report.log`: markdown suite report.
- `nprobe-refine-100k/*.log`: recall, latency, and storage logs.

`nprobe-refine-100k/truth-100k-k10.json` is regenerable truth-cache data and is
intentionally not committed.

## Key Result Lines

Suite status:

```text
[suite:task124-tq-nprobe-refine-100k-suite] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Recall:

```text
nprobe=56 recall@k=0.9990 ndcg@k=1.0000
nprobe=58 recall@k=0.9990 ndcg@k=1.0000
nprobe=60 recall@k=1.0000 ndcg@k=1.0000
nprobe=62 recall@k=1.0000 ndcg@k=1.0000
nprobe=64 recall@k=1.0000 ndcg@k=1.0000
```

Latency:

```text
nprobe=56 mean=8.26 ms p50=8.26 ms p95=8.57 ms p99=9.31 ms
nprobe=58 mean=8.29 ms p50=8.24 ms p95=8.59 ms p99=8.75 ms
nprobe=60 mean=8.52 ms p50=8.44 ms p95=8.88 ms p99=9.16 ms
nprobe=62 mean=9.06 ms p50=9.07 ms p95=9.28 ms p99=9.51 ms
nprobe=64 mean=9.28 ms p50=9.24 ms p95=9.52 ms p99=9.71 ms
```

TQ scorer counters:

```text
nprobe=56 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
nprobe=58 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
nprobe=60 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
nprobe=62 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
nprobe=64 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
```

Storage:

```text
ec_ivf index=100.8 MiB per_row=1057.2 B
```
