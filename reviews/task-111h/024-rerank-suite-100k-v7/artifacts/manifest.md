# Artifact Manifest

Task bucket: `reviews/task-111h/`
Packet: `reviews/task-111h/024-rerank-suite-100k-v7/`
Head SHA: `bc95e5f761c96b64f4a9bf594e074888981af8fe`
Timestamp: `2026-06-20T14:44:56Z`
Branch: `bench-ivf-111g-115-attribution`

## Environment

- PostgreSQL: PG18 pgrx socket at `/home/peter/.pgrx`, port `28818`
- Scratch database: `task111h_rerank_100k_v7`
- Extension install log: `artifacts/install-ecaz-pg18-release.log`
- Build log: `artifacts/cargo-build-release-pg18.log`
- Corpus: `data/staged-current/ec_real_100k_corpus.tsv`
- Queries: `data/staged-current/ec_real_100k_queries.tsv`
- Corpus manifest: `data/staged-current/ec_real_100k_manifest.json`
- Fixture shape: real 100k corpus, dim `1536`, `k=10`, 200 queries, seed `42`
- IVF reloptions shared by all benchmarked prefixes: `storage_format=coarse_rerank`, `coarse_bits=1`, `nlists=256`, `nprobe=32`, `training_sample_rows=10000`
- Run surface: isolated one-index-per-table prefixes. Continuation chunks recreated the scratch database between chunks after the initial ENOSPC event.

Corpus, query, and truth-cache files are not committed per repository packet rules. The generated truth caches were left untracked:

- `artifacts/suite/truth-100k-k10.json`
- `artifacts/suite-rabitq4-cont/truth-100k-k10.json`
- `artifacts/suite-rabitq8-cont/truth-100k-k10.json`
- `artifacts/suite-turboquant-cont/truth-100k-k10.json`

## Suite Configs

- `artifacts/task111h-100k-rerank-format-width-v7-suite.json`: main matrix for source f32, index f16, RaBitQ4, RaBitQ8, and TurboQuant.
- `artifacts/task111h-100k-rabitq4-cont-v7-suite.json`: continuation for RaBitQ4 widths 64, 128, and 256 after the main suite hit ENOSPC.
- `artifacts/task111h-100k-rabitq8-cont-v7-suite.json`: clean continuation for RaBitQ8 widths 32, 64, 128, and 256.
- `artifacts/task111h-100k-turboquant-cont-v7-suite.json`: clean continuation for TurboQuant widths 32, 64, 128, and 256.

## Commands

Build and install:

```text
cargo build --release --no-default-features --features pg18
cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18
```

Main suite:

```text
target/release/ecaz bench suite audit --config reviews/task-111h/024-rerank-suite-100k-v7/artifacts/task111h-100k-rerank-format-width-v7-suite.json
target/release/ecaz bench suite dry-run --config reviews/task-111h/024-rerank-suite-100k-v7/artifacts/task111h-100k-rerank-format-width-v7-suite.json --artifact-dir reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite --database task111h_rerank_100k_v7 --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite-dry-run-manifest.json --log-file reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite-dry-run.log
target/release/ecaz bench suite run --config reviews/task-111h/024-rerank-suite-100k-v7/artifacts/task111h-100k-rerank-format-width-v7-suite.json --artifact-dir reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite --database task111h_rerank_100k_v7 --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite-manifest.json --results-output reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results.jsonl --log-file reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite-run.log
target/release/ecaz bench suite status --manifest reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite-manifest.json --database task111h_rerank_100k_v7 --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite-status-after-enospc.log
target/release/ecaz bench suite report --manifest reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite-manifest.json --results-output reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results-report-after-enospc.jsonl --database task111h_rerank_100k_v7 --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/024-rerank-suite-100k-v7/artifacts/suite-report-after-enospc.log
```

Continuation suites used the same `bench suite audit`, `dry-run`, `run`, `status`, and `report` surfaces with the continuation configs above and these result outputs:

- `artifacts/results-rabitq4-cont.jsonl`
- `artifacts/results-rabitq4-cont-report.jsonl`
- `artifacts/results-rabitq8-cont.jsonl`
- `artifacts/results-rabitq8-cont-report.jsonl`
- `artifacts/results-turboquant-cont.jsonl`
- `artifacts/results-turboquant-cont-report.jsonl`

## Status

- Main suite status after ENOSPC: `completed=39 failed=6 skipped=0 dry_run=0 missing_artifacts=0 stale=36`, recorded in `artifacts/suite-status-after-enospc.log`.
- RaBitQ4 continuation status: `completed=13 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`, recorded in `artifacts/suite-rabitq4-cont-status.log`.
- RaBitQ8 continuation status: `completed=17 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`, recorded in `artifacts/suite-rabitq8-cont-status.log`.
- TurboQuant continuation status: `completed=17 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`, recorded in `artifacts/suite-turboquant-cont-status.log`.

The review request cites source f32, f16, and RaBitQ4 width 32 from `results-report-after-enospc.jsonl`; RaBitQ4 widths 64/128/256 from `results-rabitq4-cont-report.jsonl`; RaBitQ8 from `results-rabitq8-cont-report.jsonl`; and TurboQuant from `results-turboquant-cont-report.jsonl`.

## Key Results

See `artifacts/summary-nprobe32.md` for the compact result table. The main packet-level observations are:

- No f16 result in this 100k v7 packet supports a 150 ms latency claim. At nprobe=32, f16 p50 ranges from 4.36 ms to 14.3 ms and p99 ranges from 7.90 ms to 49.3 ms.
- Source f32 reaches recall@10 0.9285-0.9350 at nprobe=32 and 0.9875-0.9990 at nprobe=200 with a 24.6 MiB ec_ivf index.
- Persisted index f16 reaches source-like recall, but the measured ec_ivf index is 323.7-342.0 MiB and tails worsen with larger widths.
- RaBitQ4 reaches only 0.8910-0.8945 recall@10 at nprobe=32 and 0.9330-0.9380 at nprobe=200.
- RaBitQ8 improves compact quantized recall to 0.9010-0.9060 at nprobe=32 and 0.9455-0.9525 at nprobe=200, but uses a 177.4-195.4 MiB ec_ivf index.
- TurboQuant reaches 0.9040-0.9075 recall@10 at nprobe=32 and 0.9525-0.9565 at nprobe=200, with a 101.8-121.8 MiB ec_ivf index.

## Artifact Inventory

Root packet artifacts:

- `artifacts/manifest.md`
- `artifacts/summary-nprobe32.md`
- `artifacts/cargo-build-release-pg18.log`
- `artifacts/install-ecaz-pg18-release.log`
- `artifacts/db-exists.log`
- `artifacts/create-db.log`
- `artifacts/create-extension.log`
- `artifacts/db-sizes-after-enospc.log`
- `artifacts/postgres-largest-relations-after-enospc.log`
- `artifacts/drop-task111h-v7-db-after-enospc.log`
- `artifacts/recreate-task111h-v7-db.log`
- `artifacts/recreate-extension-task111h-v7.log`
- `artifacts/recreated-db-extensions.log`
- `artifacts/drop-task111h-v7-db-after-rabitq4-cont.log`
- `artifacts/recreate-task111h-v7-db-after-rabitq4-cont.log`
- `artifacts/recreate-extension-task111h-v7-after-rabitq4-cont.log`
- `artifacts/drop-task111h-v7-db-after-rabitq8-cont.log`
- `artifacts/recreate-task111h-v7-db-after-rabitq8-cont.log`
- `artifacts/recreate-extension-task111h-v7-after-rabitq8-cont.log`
- all suite config, suite manifest, suite status, suite report, and results JSONL files listed above.

Per-step logs:

- `artifacts/suite/*.log`, excluding the untracked truth cache, contains the main suite step logs.
- `artifacts/suite-rabitq4-cont/*.log`, excluding the untracked truth cache, contains the RaBitQ4 continuation step logs.
- `artifacts/suite-rabitq8-cont/*.log`, excluding the untracked truth cache, contains the RaBitQ8 continuation step logs.
- `artifacts/suite-turboquant-cont/*.log`, excluding the untracked truth cache, contains the TurboQuant continuation step logs.

## Non-Claims

This packet is not the final Task 111h decision table. It does not cover the required full 10k/50k/100k/1M matrix, table-owned persisted compact payload storage, cold/remote storage behavior, legacy `0x2A` sidecar attribution, or complete payload-byte/page-read/stage-timing counters.
