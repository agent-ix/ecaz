# Task 124 Packet 015 Artifact Manifest

- head SHA: `3be1ba32e94c2c33a4b222ee7a7271933b94e026`
- task bucket: `reviews/task-124`
- packet path: `reviews/task-124/015-tq-phase6-local-cache-evict`
- lane: Task 124 Phase 6 IO-sensitive validation
- fixture: `ec_real_100k`
- access method: `ec_ivf`
- storage format: `coarse_rerank`
- baseline: `rerank_placement=source`, `rerank_format=f32`, `rerank_width=100`
- TurboQuant: `rerank_placement=index`, `rerank_format=turboquant`, `rerank_width=75`, `rerank_group_width=50`, `stage2_final_rerank_width=15`
- run surface: isolated one-index-per-table prefixes, local macOS PG18
- cache state: `local_macos_relation_f_nocache`
- timestamp: 2026-06-29

## Review directive

This packet responds to `reviews/task-124/011-tq-selected-payload-slab/feedback/2026-06-29-01-reviewer.md`, which directed the next packet to be either Phase 2 structural work or a Shelve decision backed by Phase 6 IO-sensitive 100k validation. Packets 012-014 tried the named structural slices and reverted them after negative A/B evidence; this packet supplies the required Phase 6 100k validation.

## Commands

```text
target/release/ecaz bench suite audit --config reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/task124-tq-phase6-local-cache-evict-100k-suite.json
target/release/ecaz bench suite dry-run --config reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/task124-tq-phase6-local-cache-evict-100k-suite.json --manifest-output reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/suite-dry-run-manifest.json
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/task124-tq-phase6-local-cache-evict-100k-suite.json --manifest-output reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/suite-manifest-r2.json --results-output reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/results-r2.jsonl
target/release/ecaz --log-file reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/suite-status-r2.log bench suite status --manifest reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/suite-manifest-r2.json
target/release/ecaz --log-file reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/suite-report-r2.log bench suite report --manifest reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/suite-manifest-r2.json --results-output reviews/task-124/015-tq-phase6-local-cache-evict/artifacts/report-results-r2.jsonl
target/release/ecaz --host /Users/peter/.pgrx --port 28818 dev evict-relation-cache --prefix task124_phase6_f32_100k
target/release/ecaz --host /Users/peter/.pgrx --port 28818 dev evict-relation-cache --prefix task124_phase6_tq_100k
```

## Artifact inventory

- `task124-tq-phase6-local-cache-evict-100k-suite.json`: suite config.
- `suite-dry-run-manifest.json`: dry-run manifest.
- `suite-manifest-r2.json`: completed suite manifest.
- `results-r2.jsonl`: parsed suite results.
- `report-results-r2.jsonl`: parsed report output.
- `suite-status-r2.log`: completed/failed/skipped status for all 10 steps.
- `suite-report-r2.log`: markdown suite report with parsed results.
- `cache-evict-summary.md`: durable summary of explicit f32 and TQ relation-cache eviction reruns.
- `local-cache-evict-100k/*.log`: packet-local load, storage, latency, recall, and raw-step artifacts.

`local-cache-evict-100k/truth-100k-k10.json` is regenerable truth-cache data and is intentionally not committed.

## Key result lines

Suite status:

```text
[suite:task124-tq-phase6-local-cache-evict-100k-suite] completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Storage:

```text
f32 total=1.6 GiB indexes=24.6 MiB ec_ivf_index=22.5 MiB per_row_index=235.8 B
tq  total=1.7 GiB indexes=103.0 MiB ec_ivf_index=100.8 MiB per_row_index=1057.2 B
```

Latency:

```text
f32 nprobe=32 mean=6.39 ms p50=5.74 ms p95=9.53 ms p99=13.8 ms
tq  nprobe=32 mean=7.37 ms p50=6.76 ms p95=10.9 ms p99=14.3 ms
f32 nprobe=64 mean=9.20 ms p50=9.01 ms p95=11.5 ms p99=11.8 ms
tq  nprobe=64 mean=9.44 ms p50=9.24 ms p95=9.98 ms p99=12.8 ms
```

Recall:

```text
f32 nprobe=32 recall@k=0.9730 ndcg@k=0.9969
tq  nprobe=32 recall@k=0.9730 ndcg@k=0.9969
f32 nprobe=64 recall@k=1.0000 ndcg@k=1.0000
tq  nprobe=64 recall@k=1.0000 ndcg@k=1.0000
```

Kernel counters:

```text
tq nprobe=32 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
tq nprobe=64 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
```

Cache eviction:

```text
f32 cache_evict_summary database=tqvector_bench dry_run=false relations=5 files=10 bytes=1690509312
tq  cache_evict_summary database=tqvector_bench dry_run=false relations=5 files=10 bytes=1772642304
```
