# Task 124 Packet 020 Artifact Manifest

- head SHA before packet: `c197ca7af0fe714165fa8774d49cf95baec6b24a`
- task bucket: `reviews/task-124`
- packet path: `reviews/task-124/020-tq-borrowed-score-buffer`
- lane: TQ speed attempt, negative result
- fixture: `ec_real_100k`
- access method: `ec_ivf`
- storage format: `coarse_rerank`
- TQ config: `rerank_placement=index`, `rerank_format=turboquant`,
  `rerank_width=75`, `rerank_group_width=50`,
  `stage2_final_rerank_width=15`
- run surface: isolated one-index-per-table prefix
  `task124_tq_bscore_w75_g50_100k`
- date: 2026-06-29
- timestamp: 2026-06-30T01:57:38Z

## Code Attempt

Temporary diff:

- `discarded-borrowed-score-buffer.diff`

The attempted speed slice added an index-side TurboQuant borrowed-payload batch
scorer that writes directly into the caller's score buffer, avoiding the
temporary `estimates` vector and copy/negate loop in
`RerankPayloadCodec::score_payload_refs_batch`.

Result: do not land. The source tree was reverted after measurement.

## Validation Commands

```text
cargo fmt --check
cargo check -p ecaz --lib --no-default-features --features pg18
cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
target/release/ecaz bench suite audit --config reviews/task-124/020-tq-borrowed-score-buffer/artifacts/task124-tq-borrowed-score-buffer-100k-suite.json
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-124/020-tq-borrowed-score-buffer/artifacts/task124-tq-borrowed-score-buffer-100k-suite.json --manifest-output reviews/task-124/020-tq-borrowed-score-buffer/artifacts/suite-manifest.json --results-output reviews/task-124/020-tq-borrowed-score-buffer/artifacts/results.jsonl
target/release/ecaz --log-file reviews/task-124/020-tq-borrowed-score-buffer/artifacts/suite-status.log bench suite status --manifest reviews/task-124/020-tq-borrowed-score-buffer/artifacts/suite-manifest.json
target/release/ecaz --log-file reviews/task-124/020-tq-borrowed-score-buffer/artifacts/suite-report.log bench suite report --manifest reviews/task-124/020-tq-borrowed-score-buffer/artifacts/suite-manifest.json --results-output reviews/task-124/020-tq-borrowed-score-buffer/artifacts/report-results.jsonl
```

## Artifact Inventory

- `task124-tq-borrowed-score-buffer-100k-suite.json`: suite config.
- `suite-manifest.json`: completed suite manifest.
- `results.jsonl`: parsed suite results.
- `report-results.jsonl`: parsed report output.
- `suite-status.log`: 4-step suite status.
- `suite-report.log`: markdown suite report.
- `borrowed-score-buffer-100k/*.log`: load, recall, latency, and storage logs.
- `discarded-borrowed-score-buffer.diff`: reverted code attempt.

`borrowed-score-buffer-100k/truth-100k-k10.json` is regenerable truth-cache data
and is intentionally not committed.

## Key Result Lines

Suite status:

```text
[suite:task124-tq-borrowed-score-buffer-100k-suite] completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Recall:

```text
nprobe=32 recall@k=0.9730 ndcg@k=0.9969
nprobe=64 recall@k=1.0000 ndcg@k=1.0000
```

Latency:

```text
nprobe=32 mean=4.89 ms p50=4.86 ms p95=5.41 ms p99=5.76 ms
nprobe=64 mean=9.10 ms p50=9.05 ms p95=9.48 ms p99=9.60 ms
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
