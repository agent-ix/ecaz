# Task 124 / 035 Post-Scorer Product Suite Manifest

- head SHA: `c79603ce1b0f2677299920b8d52eef2dfc8f3553e`
- task bucket: `reviews/task-124/035-post-scorer-product-suite`
- lane: local PG18 release `ec_ivf`
- fixture: staged real 10k / 50k / 100k corpora under `data/staged-current/`
- storage format: `coarse_rerank`
- rerank modes:
  - f32/source baseline: `coarse_format=rabitq`, `coarse_bits=1`,
    `rerank_placement=source`, `rerank_format=f32`, `rerank_width=100`,
    `stage2_final_rerank_width=0`
  - TQ final15: `coarse_format=rabitq`, `coarse_bits=1`,
    `rerank_placement=index`, `rerank_format=turboquant`,
    `rerank_width=75`, `rerank_group_width=50`,
    `stage2_final_rerank_width=15`
- timestamp: 2026-06-30
- isolated one-index-per-table: yes, separate prefixes per scale and variant

## Build / Install

Commands:

```sh
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
```

Key result lines:

```text
cargo build --release -p ecaz: Finished `release` profile [optimized]
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config: Finished installing ecaz
```

## Suite Config

### `task124-post-scorer-product-suite.json`

Copied from packet 026's discriminator suite config. The run command overrode
the config artifact directory so all new logs/results landed under this packet:
`reviews/task-124/035-post-scorer-product-suite/artifacts/post-scorer-suite`.

## Artifacts

### `suite-audit.log`

Command:

```sh
./target/release/ecaz bench suite audit --config reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/task124-f32-vs-tq-nprobe60-10-50-100-suite.json --log-file reviews/task-124/035-post-scorer-product-suite/artifacts/suite-audit.log
```

Key result line:

```text
[suite:task124-f32-vs-tq-nprobe60-10-50-100-suite] audit passed: 24 steps
```

### `suite-run.log`

Initial failed connection attempt. It executed zero suite steps because the PG
host/port were omitted.

Key result lines:

```text
connecting to Postgres database "tqvector_bench"
both host and hostaddr are missing
```

### `suite-run-r2.log`

Authoritative run.

Command:

```sh
./target/release/ecaz bench suite run --config reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/artifacts/task124-f32-vs-tq-nprobe60-10-50-100-suite.json --artifact-dir reviews/task-124/035-post-scorer-product-suite/artifacts/post-scorer-suite --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/035-post-scorer-product-suite/artifacts/suite-run-r2.log
```

Key result line:

```text
[suite:task124-f32-vs-tq-nprobe60-10-50-100-suite] wrote reviews/task-124/035-post-scorer-product-suite/artifacts/post-scorer-suite/results.jsonl
```

### `suite-status.log`

Command:

```sh
./target/release/ecaz bench suite status --manifest reviews/task-124/035-post-scorer-product-suite/artifacts/post-scorer-suite/suite-manifest.json --log-file reviews/task-124/035-post-scorer-product-suite/artifacts/suite-status.log
```

Key result line:

```text
[suite:task124-f32-vs-tq-nprobe60-10-50-100-suite] completed=24 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

### `suite-report.log`

Command:

```sh
./target/release/ecaz bench suite report --manifest reviews/task-124/035-post-scorer-product-suite/artifacts/post-scorer-suite/suite-manifest.json --log-file reviews/task-124/035-post-scorer-product-suite/artifacts/suite-report.log
```

Key result rows:

```text
10k f32/source: recall@10=1.0000, ndcg@10=1.0000, p50=1.14 ms, p95=1.26 ms, p99=1.36 ms, index=2.9 MiB
10k TQ final15: recall@10=1.0000, ndcg@10=1.0000, p50=1.04 ms, p95=1.19 ms, p99=1.35 ms, index=10.9 MiB
50k f32/source: recall@10=1.0000, ndcg@10=1.0000, p50=4.13 ms, p95=4.33 ms, p99=4.41 ms, index=11.6 MiB
50k TQ final15: recall@10=0.9980, ndcg@10=1.0000, p50=4.13 ms, p95=4.48 ms, p99=4.60 ms, index=50.9 MiB
100k f32/source: recall@10=1.0000, ndcg@10=1.0000, p50=8.22 ms, p95=8.48 ms, p99=9.24 ms, index=22.5 MiB
100k TQ final15: recall@10=1.0000, ndcg@10=1.0000, p50=8.30 ms, p95=9.41 ms, p99=9.68 ms, index=100.8 MiB
```

### `post-scorer-suite/results.jsonl`

Normalized result rows emitted by `ecaz bench suite`.

Key TQ scorer counter rows:

```text
10k TQ scorer: quant=turboquant, isa=neon, candidates=7500, scalar_candidates=0, elapsed_ms=1.779246
50k TQ scorer: quant=turboquant, isa=neon, candidates=7500, scalar_candidates=0, elapsed_ms=1.788211
100k TQ scorer: quant=turboquant, isa=neon, candidates=7500, scalar_candidates=0, elapsed_ms=1.804748
```

## Omitted From Commit

The generated `truth-10k-k10.json`, `truth-50k-k10.json`, and
`truth-100k-k10.json` files are packet-local runtime artifacts but are
regenerable truth-cache data and are not intended to be committed.
