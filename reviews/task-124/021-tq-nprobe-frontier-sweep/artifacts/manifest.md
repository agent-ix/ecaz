# Task 124 Packet 021 Artifact Manifest

- head SHA before packet: `9f0445236f955142a50a634d6c5fb97a10855b0e`
- task bucket: `reviews/task-124`
- packet path: `reviews/task-124/021-tq-nprobe-frontier-sweep`
- lane: TQ speed diagnostic
- fixture: `ec_real_100k`
- access method: `ec_ivf`
- storage format: `coarse_rerank`
- TQ config: `rerank_placement=index`, `rerank_format=turboquant`,
  `rerank_width=75`, `rerank_group_width=50`,
  `stage2_final_rerank_width=15`
- run surface: shared-table reuse of packet `020` isolated prefix
  `task124_tq_bscore_w75_g50_100k`
- date: 2026-06-29
- timestamp: 2026-06-30T02:05:49Z

## Setup Note

Packet `020` temporarily installed a discarded score-buffer code attempt. Before
running this packet, the source tree was reverted and the extension was rebuilt
and reinstalled:

```text
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
```

This packet therefore measures the current branch source, not the discarded
packet `020` code.

## Commands

```text
target/release/ecaz bench suite audit --config reviews/task-124/021-tq-nprobe-frontier-sweep/artifacts/task124-tq-nprobe-frontier-100k-suite.json
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-124/021-tq-nprobe-frontier-sweep/artifacts/task124-tq-nprobe-frontier-100k-suite.json --manifest-output reviews/task-124/021-tq-nprobe-frontier-sweep/artifacts/suite-manifest.json --results-output reviews/task-124/021-tq-nprobe-frontier-sweep/artifacts/results.jsonl
target/release/ecaz --log-file reviews/task-124/021-tq-nprobe-frontier-sweep/artifacts/suite-status.log bench suite status --manifest reviews/task-124/021-tq-nprobe-frontier-sweep/artifacts/suite-manifest.json
target/release/ecaz --log-file reviews/task-124/021-tq-nprobe-frontier-sweep/artifacts/suite-report.log bench suite report --manifest reviews/task-124/021-tq-nprobe-frontier-sweep/artifacts/suite-manifest.json --results-output reviews/task-124/021-tq-nprobe-frontier-sweep/artifacts/report-results.jsonl
```

## Artifact Inventory

- `task124-tq-nprobe-frontier-100k-suite.json`: suite config.
- `suite-manifest.json`: completed suite manifest.
- `results.jsonl`: parsed suite results.
- `report-results.jsonl`: parsed report output.
- `suite-status.log`: 3-step suite status.
- `suite-report.log`: markdown suite report.
- `nprobe-frontier-100k/*.log`: recall, latency, and storage logs.

`nprobe-frontier-100k/truth-100k-k10.json` is regenerable truth-cache data and
is intentionally not committed.

## Key Result Lines

Suite status:

```text
[suite:task124-tq-nprobe-frontier-100k-suite] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Recall:

```text
nprobe=32 recall@k=0.9730 ndcg@k=0.9969
nprobe=40 recall@k=0.9910 ndcg@k=0.9992
nprobe=48 recall@k=0.9940 ndcg@k=0.9996
nprobe=56 recall@k=0.9990 ndcg@k=1.0000
nprobe=64 recall@k=1.0000 ndcg@k=1.0000
```

Latency:

```text
nprobe=32 mean=4.83 ms p50=4.83 ms p95=5.49 ms p99=5.80 ms
nprobe=40 mean=5.95 ms p50=5.94 ms p95=6.48 ms p99=6.67 ms
nprobe=48 mean=7.12 ms p50=7.09 ms p95=7.66 ms p99=8.07 ms
nprobe=56 mean=8.07 ms p50=8.03 ms p95=8.34 ms p99=8.63 ms
nprobe=64 mean=8.94 ms p50=8.88 ms p95=9.12 ms p99=9.34 ms
```

TQ scorer counters:

```text
nprobe=32 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
nprobe=40 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
nprobe=48 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
nprobe=56 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
nprobe=64 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
```

Storage:

```text
ec_ivf index=100.8 MiB per_row=1057.2 B
```
