# Artifact Manifest

Task bucket: `reviews/task-111h`

Packet: `reviews/task-111h/021-rerank-suite-1m-shared`

Head SHA: `0e0d774e1db93a40373481139b6de4c7a19ab679`

Branch: `bench-ivf-111g-115-attribution`

Captured: 2026-06-20

## Host And Database

- PostgreSQL: 18.3 on x86_64-pc-linux-gnu.
- Host/socket/port: `/home/peter/.pgrx`, port `28818`.
- Database: `task111h_rerank_1m`.
- Host settings from `artifacts/suite/precheck-host.log`: `shared_buffers=128MB`, `work_mem=4MB`, `maintenance_work_mem=64MB`, `effective_cache_size=4GB`.

## Corpus

- Dataset: Qdrant `dbpedia-entities-openai3-text-embedding-3-large-1536-1M`.
- Prepared profile: `ec_real_ann_benchmarks_anchor`.
- Dimension: 1536.
- Corpus rows: 990000.
- Query rows: 10000.
- Sort/selection: `_id ascending lexicographic`; corpus rows 0-989999, query rows 990000-999999.
- Fetch manifest path: `data/benchmark-profile-inputs/dbpedia-openai3-1m-fetch/ecaz_fetch_manifest.json`.
- Fetch manifest SHA256: `453a6924c7306403fa74256896935848c66c6a110f7a6bfefa45d9e2ba280d34`.
- Prepared manifest path: `data/benchmark-profile-inputs/dbpedia-openai3-1m-staged/ec_real_ann_benchmarks_anchor_manifest.json`.
- Prepared manifest SHA256: `86f8f9bd12a53c19f0244f4c05c45a4665799ccd426f86140ae4d0e4888dfbc9`.
- Corpus/query TSVs and parquet shards are not committed per AGENTS.md; their paths and SHA256 chunk metadata are in the prepared/fetch manifests.

## Suite

- Suite config: `artifacts/task111h-1m-rerank-format-width-shared-suite.json`.
- Config SHA256: `8b7de524e34d23fcc4fc80b5ac0267640816b885f0f4f09f9591077b3bbbaf9e`.
- Suite manifest: `artifacts/suite-manifest.json`.
- Suite manifest SHA256: `33f1866f0c351bce1fbbe295e19771314be2a8342a17ebf810ed8a33e62f8169`.
- Results JSONL: `artifacts/results.jsonl`.
- Results JSONL SHA256: `ae7f7776e0a5bbe0a7e6b8ebc43fada68771c56894709ec44d162ff59440bd8f`.
- Report replay JSONL: `artifacts/results-report.jsonl`.
- Report replay JSONL SHA256: `ae7f7776e0a5bbe0a7e6b8ebc43fada68771c56894709ec44d162ff59440bd8f`.
- Status log: `artifacts/suite-status.log`.
- Report log: `artifacts/suite-report.log`.
- Runner result: 124 completed, 0 failed, 0 skipped, 0 dry-run, 0 missing artifacts, 0 stale.

## Commands

Build/checkpoint preparation:

```bash
cargo build --release --no-default-features --features pg18
cargo build --release -p ecaz-cli
```

Suite validation:

```bash
target/release/ecaz bench suite audit --config reviews/task-111h/021-rerank-suite-1m-shared/artifacts/task111h-1m-rerank-format-width-shared-suite.json --artifact-dir reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite --database task111h_rerank_1m --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-manifest.json --log-file reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-audit-after-cli-build.log
target/release/ecaz bench suite dry-run --config reviews/task-111h/021-rerank-suite-1m-shared/artifacts/task111h-1m-rerank-format-width-shared-suite.json --artifact-dir reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite --database task111h_rerank_1m --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-dry-run-manifest.json --log-file reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-dry-run.log
```

Database setup:

```bash
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "SELECT 1 FROM pg_database WHERE datname = 'task111h_rerank_1m';" --log-output reviews/task-111h/021-rerank-suite-1m-shared/artifacts/db-exists.log
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "CREATE DATABASE task111h_rerank_1m;" --log-output reviews/task-111h/021-rerank-suite-1m-shared/artifacts/create-db.log
target/release/ecaz dev sql --pg 18 --db task111h_rerank_1m --socket-dir /home/peter/.pgrx --raw --sql "CREATE EXTENSION IF NOT EXISTS ecaz;" --log-output reviews/task-111h/021-rerank-suite-1m-shared/artifacts/create-extension-setup.log
```

Suite run:

```bash
target/release/ecaz bench suite run --config reviews/task-111h/021-rerank-suite-1m-shared/artifacts/task111h-1m-rerank-format-width-shared-suite.json --artifact-dir reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite --database task111h_rerank_1m --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-manifest.json --results-output reviews/task-111h/021-rerank-suite-1m-shared/artifacts/results.jsonl --log-file reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-run.log
```

Suite status/report:

```bash
target/release/ecaz bench suite status --manifest reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-manifest.json --database task111h_rerank_1m --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-status.log
target/release/ecaz bench suite report --manifest reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-manifest.json --results-output reviews/task-111h/021-rerank-suite-1m-shared/artifacts/results-report.jsonl --database task111h_rerank_1m --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/021-rerank-suite-1m-shared/artifacts/suite-report.log
```

## Surface Isolation

This packet used shared-table surfaces to control load time while still keeping one active measured index at a time. Each cell has `DROP INDEX IF EXISTS ... before`, `load`, `recall`, `latency`, `storage`, and `DROP INDEX ... after` steps. The source table was reused after the first source-f32 cell, and later load logs explicitly show corpus/query chunks skipped as already loaded.

## Notes

- A stale CLI dry-run was caught before the suite run because load commands lacked `--index-name`; `cargo build --release -p ecaz-cli` fixed the runner binary used here.
- Exact truth was generated once during the suite and reused by recall steps. The local `truth-1m-k10.json` cache is regenerable and was deleted before committing this packet.
- Diagnostic active-build snapshots for long f16/TQ builds are stored in `artifacts/suite/diagnostic-*.log`.
