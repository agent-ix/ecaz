# Task 124 Packet 018 Artifact Manifest

- head SHA before packet: `1eeedac03de8f9bbd1f6cb9dd74529debd8c34dd`
- task bucket: `reviews/task-124`
- packet path: `reviews/task-124/018-tq-selected-index-vector`
- lane: TQ speed attempt, negative result
- fixture: `ec_real_100k`
- access method: `ec_ivf`
- storage format: `coarse_rerank`
- TQ config: `rerank_placement=index`, `rerank_format=turboquant`, `rerank_width=75`, `rerank_group_width=50`, `stage2_final_rerank_width=15`
- run surface: isolated one-index-per-table prefix `task124_tq_vecidx_w75_g50_100k`
- date: 2026-06-29

## Code Attempt

Temporary diff:

- `discarded-selected-index-vector.diff`

The attempted speed slice replaced the selected-payload slab's inner
`HashMap<ItemPointer, usize>` with a compact vector of `(heap_tid, payload_index)`
entries. The idea was to reduce allocation/hash overhead for the small selected
payload sets used by the TQ stage-2 path.

Result: do not land. The source tree was reverted after measurement.

## Commands

```text
cargo fmt --check
cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
target/release/ecaz bench suite audit --config reviews/task-124/018-tq-selected-index-vector/artifacts/task124-tq-selected-index-vector-100k-suite.json
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-124/018-tq-selected-index-vector/artifacts/task124-tq-selected-index-vector-100k-suite.json --manifest-output reviews/task-124/018-tq-selected-index-vector/artifacts/suite-manifest.json --results-output reviews/task-124/018-tq-selected-index-vector/artifacts/results.jsonl
target/release/ecaz --log-file reviews/task-124/018-tq-selected-index-vector/artifacts/suite-status.log bench suite status --manifest reviews/task-124/018-tq-selected-index-vector/artifacts/suite-manifest.json
target/release/ecaz --log-file reviews/task-124/018-tq-selected-index-vector/artifacts/suite-report.log bench suite report --manifest reviews/task-124/018-tq-selected-index-vector/artifacts/suite-manifest.json --results-output reviews/task-124/018-tq-selected-index-vector/artifacts/report-results.jsonl
```

## Artifact Inventory

- `task124-tq-selected-index-vector-100k-suite.json`: suite config.
- `suite-manifest.json`: completed suite manifest.
- `results.jsonl`: parsed suite results.
- `report-results.jsonl`: parsed report output.
- `suite-status.log`: 4-step suite status.
- `suite-report.log`: markdown suite report.
- `selected-index-vector-100k/*.log`: load, recall, latency, and storage logs.
- `discarded-selected-index-vector.diff`: reverted code attempt.

`selected-index-vector-100k/truth-100k-k10.json` is regenerable truth-cache data and is intentionally not committed.

## Key Result Lines

Suite status:

```text
[suite:task124-tq-selected-index-vector-100k-suite] completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Recall:

```text
nprobe=32 recall@k=0.9730 ndcg@k=0.9969
nprobe=64 recall@k=1.0000 ndcg@k=1.0000
```

Latency:

```text
nprobe=32 mean=5.08 ms p50=5.03 ms p95=5.59 ms p99=5.93 ms
nprobe=64 mean=9.41 ms p50=9.37 ms p95=9.66 ms p99=9.77 ms
```

Packet 011 baseline for the same TQ shape:

```text
nprobe=32 mean=4.85 ms p50=4.83 ms p95=5.35 ms p99=5.55 ms
nprobe=64 mean=8.95 ms p50=8.91 ms p95=9.14 ms p99=9.25 ms
```

TQ scorer counters:

```text
nprobe=32 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
nprobe=64 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
```

Storage:

```text
ec_ivf index=100.8 MiB per_row=1057.2 B
```
