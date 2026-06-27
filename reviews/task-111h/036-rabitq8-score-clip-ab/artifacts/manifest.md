# Artifact Manifest

Task bucket: `reviews/task-111h/`
Packet: `reviews/task-111h/036-rabitq8-score-clip-ab/`
Head SHA: `53caaa57245763970452425c55a4738a18bc93fd`
Code commit under measurement: `26de9a0c6a72a92646fe275d3883889008a82e58`
Timestamp: `2026-06-20T14:28:45-07:00`
Branch: `bench-ivf-111g-115-attribution`

## Environment

- PostgreSQL: PG18 pgrx socket at `/home/peter/.pgrx`, port `28818`
- Scratch database: `task111h_rabitq8_score_clip_ab`
- Build log: `artifacts/cargo-build-release-pg18.log`
- Install log: `artifacts/cargo-pgrx-install-pg18-release.log`
- CLI build log: `artifacts/cargo-build-release-ecaz-cli.log`
- Corpus: `data/staged-current/ec_real_100k_corpus.tsv`
- Queries: `data/staged-current/ec_real_100k_queries.tsv`
- Corpus manifest: `data/staged-current/ec_real_100k_manifest.json`
- Fixture shape: real 100k corpus, dim `1536`, `k=10`, 200 queries, seed `42`
- Run surface: isolated one-index-per-table prefixes. After the initial ENOSPC event, each remaining score/clip variant ran after a scratch database drop/recreate.

Corpus, query, and truth-cache files are not committed per repository packet rules. The suite generated these truth-cache paths during recall, and they were removed before staging the packet:

- `artifacts/suite/truth-100k-k10.json`
- `artifacts/suite-cont-est-c3/truth-100k-k10.json`
- `artifacts/suite-cont-ls-c3/truth-100k-k10.json`
- `artifacts/suite-cont-est-c4/truth-100k-k10.json`
- `artifacts/suite-cont-ls-c4/truth-100k-k10.json`

## Suite Config

- `artifacts/task111h-100k-rabitq8-score-clip-ab-suite.json`
- Config SHA256: `a2ba3d9fa3d664a4577c217667278b678d2b69e82dc4cae1f452bfac8df2dfd2`
- Matrix:
  - `rerank_format=rabitq8`
  - `rerank_width=64`
  - `rabitq_rerank_least_squares=0|1`
  - `rabitq_rerank_clip=2|3|4`
  - recall/latency sweep `nprobe=8,16,32,64,128,200`

## Commands

Build and install:

```text
cargo build --release --no-default-features --features pg18
cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18
cargo build --release -p ecaz-cli
```

Initial suite:

```text
target/release/ecaz bench suite audit --config reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/task111h-100k-rabitq8-score-clip-ab-suite.json
target/release/ecaz bench suite run --dry-run --config reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/task111h-100k-rabitq8-score-clip-ab-suite.json --artifact-dir reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite --database task111h_rabitq8_score_clip_ab --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite-dry-run-manifest.json --log-file reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite-dry-run.log
target/release/ecaz bench suite run --config reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/task111h-100k-rabitq8-score-clip-ab-suite.json --artifact-dir reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite --database task111h_rabitq8_score_clip_ab --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite-manifest.json --results-output reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/results.jsonl --log-file reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite-run.log
target/release/ecaz bench suite status --manifest reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite-manifest.json --log-file reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite-status-after-enospc.log
target/release/ecaz bench suite report --manifest reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite-manifest.json --results-output reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/results-report-after-enospc.jsonl --log-file reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/suite-report-after-enospc.log
```

Continuation runs used the same suite config with `--only` filters and separate artifact directories:

- `artifacts/suite-cont-est-c3/`, result files `artifacts/results-cont-est-c3.jsonl` and `artifacts/results-cont-est-c3-report.jsonl`
- `artifacts/suite-cont-ls-c3/`, result files `artifacts/results-cont-ls-c3.jsonl` and `artifacts/results-cont-ls-c3-report.jsonl`
- `artifacts/suite-cont-est-c4/`, result files `artifacts/results-cont-est-c4.jsonl` and `artifacts/results-cont-est-c4-report.jsonl`
- `artifacts/suite-cont-ls-c4/`, result files `artifacts/results-cont-ls-c4.jsonl` and `artifacts/results-cont-ls-c4-report.jsonl`

Each continuation was preceded by:

```text
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "DROP DATABASE IF EXISTS task111h_rabitq8_score_clip_ab WITH (FORCE);"
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "CREATE DATABASE task111h_rabitq8_score_clip_ab;"
target/release/ecaz dev sql --pg 18 --db task111h_rabitq8_score_clip_ab --socket-dir /home/peter/.pgrx --raw --sql "CREATE EXTENSION ecaz;"
```

The final cleanup command was recorded in `artifacts/drop-db-final.log`.

## Status

- Audit: `artifacts/suite-audit.log` reports `audit passed: 25 steps`.
- Initial suite after ENOSPC: `completed=10 failed=3 skipped=0 dry_run=0 missing_artifacts=0 stale=12`, recorded in `artifacts/suite-status-after-enospc.log`.
- Estimator clip=3 continuation: `completed=4 failed=0 skipped=21 dry_run=0 missing_artifacts=0 stale=0`, recorded in `artifacts/suite-cont-est-c3-status.log`.
- Least-squares clip=3 continuation: `completed=4 failed=0 skipped=21 dry_run=0 missing_artifacts=0 stale=0`, recorded in `artifacts/suite-cont-ls-c3-status.log`.
- Estimator clip=4 continuation: `completed=4 failed=0 skipped=21 dry_run=0 missing_artifacts=0 stale=0`, recorded in `artifacts/suite-cont-est-c4-status.log`.
- Least-squares clip=4 continuation: `completed=4 failed=0 skipped=21 dry_run=0 missing_artifacts=0 stale=0`, recorded in `artifacts/suite-cont-ls-c4-status.log`.

## Key Results

See `artifacts/summary-score-clip-ab.md` for the compact table. The main packet-level observations are:

- Clip=2 did not support a strong RaBitQ8 ceiling: estimator reached recall@10 `0.9060` at nprobe32 and `0.9525` at nprobe200.
- Clip=3 materially improved recall without changing index size: estimator reached `0.9260` at nprobe32 and `0.9830` at nprobe200.
- Clip=4 improved further: estimator reached `0.9305` at nprobe32 and `0.9915` at nprobe200; least-squares reached `0.9305` at nprobe32 and `0.9920` at nprobe200.
- All six score/clip variants reported the same ec_ivf index size: `183.6 MiB`.
- Least-squares did not improve clip=2 or clip=3 in this run. At clip=4 it only improved nprobe200 by `0.0005`.

## Artifact Inventory

Root packet artifacts:

- `artifacts/manifest.md`
- `artifacts/summary-score-clip-ab.md`
- `artifacts/task111h-100k-rabitq8-score-clip-ab-suite.json`
- build/install logs: `cargo-build-release-pg18.log`, `cargo-pgrx-install-pg18-release.log`, `cargo-build-release-ecaz-cli.log`
- database setup/cleanup logs: `drop-db.log`, `create-db.log`, `create-extension.log`, all `drop-db-cont-*.log`, `create-db-cont-*.log`, `create-extension-cont-*.log`, and `drop-db-final.log`
- suite logs and reports: `suite-audit.log`, `suite-dry-run.log`, `suite-dry-run-manifest.json`, `suite-manifest.json`, `suite-run.log`, `suite-status-after-enospc.log`, `suite-report-after-enospc.log`, `results.jsonl`, `results-report-after-enospc.jsonl`
- continuation manifests, run logs, status logs, report logs, and JSONL result/report files for `est-c3`, `ls-c3`, `est-c4`, and `ls-c4`

Per-step logs:

- `artifacts/suite/*.log`, excluding the untracked truth cache, contains the initial suite step logs.
- `artifacts/suite-cont-est-c3/*.log`, excluding the untracked truth cache, contains estimator clip=3 step logs.
- `artifacts/suite-cont-ls-c3/*.log`, excluding the untracked truth cache, contains least-squares clip=3 step logs.
- `artifacts/suite-cont-est-c4/*.log`, excluding the untracked truth cache, contains estimator clip=4 step logs.
- `artifacts/suite-cont-ls-c4/*.log`, excluding the untracked truth cache, contains least-squares clip=4 step logs.

## Non-Claims

This packet is not the final Task 111h closeout. It does not cover table-owned persisted compact payload storage, cold/remote behavior, f16 storage redesign, RaBitQ4/TurboQuant score-clip equivalents, or the full 10k/50k/100k/1M decision matrix.
