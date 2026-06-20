# Artifact Manifest: Task 111h Packet 040

Head SHA: `9b6f91a8dedc124c0f27fe062285ed3c77c0b4a7`

Task bucket: `reviews/task-111h/`

Packet path: `reviews/task-111h/040-cold-cache-50k-candidates/`

Timestamp: 2026-06-20 15:40-15:58 America/Los_Angeles

Lane: local PG18, x86_64, cold-start relation-file eviction probe.

Fixture: `data/staged-current/ec_real_50k_corpus.tsv`, `data/staged-current/ec_real_50k_queries.tsv`, `data/staged-current/ec_real_50k_manifest.json`, `dim=1536`, `queries=200`, `k=10`.

Surface isolation: one prefix/table/index per candidate in one suite database (`task111h_cold_50k`). No shared-table multi-index surface was used.

Common index options: `profile=ec_ivf`, `nlists=256`, `nprobe=32`, `training_sample_rows=10000`, `storage_format=coarse_rerank`, `coarse_bits=1`, `rerank=heap_f32`.

Candidates:

| Prefix | Rerank placement | Rerank format | Width | Extra knobs |
| --- | --- | --- | --- | --- |
| `task111h040_50k_source_f32_w32` | `source` | `f32` | 32 | none |
| `task111h040_50k_index_f16_w32` | `index` | `f16` | 32 | none |
| `task111h040_50k_index_rq4_w128` | `index` | `rabitq4` | 128 | none |
| `task111h040_50k_index_rq8_c4_w64` | `index` | `rabitq8` | 64 | `rabitq_rerank_least_squares=0`, `rabitq_rerank_clip=4` |
| `task111h040_50k_index_tq_w32` | `index` | `turboquant` | 32 | none |

## Commands

Release CLI build:

```sh
env CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo build --release -p ecaz-cli
```

Database setup:

```sh
/home/peter/dev/ecaz/target/release/ecaz --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql 'DROP DATABASE IF EXISTS task111h_cold_50k WITH (FORCE);'
/home/peter/dev/ecaz/target/release/ecaz --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql 'CREATE DATABASE task111h_cold_50k;'
/home/peter/dev/ecaz/target/release/ecaz --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db task111h_cold_50k --socket-dir /home/peter/.pgrx --raw --sql 'CREATE EXTENSION ecaz;'
```

Suite audit:

```sh
/home/peter/dev/ecaz/target/release/ecaz bench suite audit --config reviews/task-111h/040-cold-cache-50k-candidates/artifacts/task111h-50k-cold-start-candidates-suite.json --database task111h_cold_50k --host /home/peter/.pgrx --port 28818
```

Suite dry-run:

```sh
/home/peter/dev/ecaz/target/release/ecaz bench suite run --config reviews/task-111h/040-cold-cache-50k-candidates/artifacts/task111h-50k-cold-start-candidates-suite.json --artifact-dir reviews/task-111h/040-cold-cache-50k-candidates/artifacts/suite --database task111h_cold_50k --host /home/peter/.pgrx --port 28818 --dry-run --manifest-output reviews/task-111h/040-cold-cache-50k-candidates/artifacts/suite-dry-run-manifest.json
```

Suite run:

```sh
/home/peter/dev/ecaz/target/release/ecaz bench suite run --config reviews/task-111h/040-cold-cache-50k-candidates/artifacts/task111h-50k-cold-start-candidates-suite.json --artifact-dir reviews/task-111h/040-cold-cache-50k-candidates/artifacts/suite --database task111h_cold_50k --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/040-cold-cache-50k-candidates/artifacts/suite-manifest.json --results-output reviews/task-111h/040-cold-cache-50k-candidates/artifacts/results.jsonl --log-file reviews/task-111h/040-cold-cache-50k-candidates/artifacts/suite-run.log
```

Suite status and report:

```sh
/home/peter/dev/ecaz/target/release/ecaz bench suite status --manifest reviews/task-111h/040-cold-cache-50k-candidates/artifacts/suite-manifest.json
/home/peter/dev/ecaz/target/release/ecaz bench suite report --results reviews/task-111h/040-cold-cache-50k-candidates/artifacts/results.jsonl --output reviews/task-111h/040-cold-cache-50k-candidates/artifacts/results-report.jsonl
```

Post-run eviction dry-run probes:

```sh
/home/peter/dev/ecaz/target/release/ecaz --database task111h_cold_50k --host /home/peter/.pgrx --port 28818 dev evict-relation-cache --prefix <candidate-prefix> --dry-run
```

## Artifact Inventory

| Artifact | Purpose | Result |
| --- | --- | --- |
| `artifacts/cargo-build-release-ecaz-cli.log` | Builds the release CLI used for the suite. | Succeeded; one pre-existing `LoadedDistributedPlacementConfig::path` dead-code warning. |
| `artifacts/drop-db.log` | Drops prior `task111h_cold_50k` database. | Succeeded. |
| `artifacts/create-db.log` | Creates fresh `task111h_cold_50k` database. | Succeeded. |
| `artifacts/create-extension.log` | Installs `ecaz` extension in the fresh database. | Succeeded. |
| `artifacts/task111h-50k-cold-start-candidates-suite.json` | Checked-in `ecaz bench suite` config. | 46 steps: precheck plus load/recall/storage/evict/latency for 5 candidates. |
| `artifacts/suite-audit.log` | Suite config audit. | `[suite:task111h-50k-cold-start-candidates] audit passed: 46 steps`. |
| `artifacts/suite-dry-run.log` | Dry-run command trace. | Succeeded. |
| `artifacts/suite-dry-run-manifest.json` | Dry-run manifest. | Captures the planned 46-step command sequence. |
| `artifacts/suite-run.log` | Actual suite command trace. | Succeeded and wrote `results.jsonl` / `suite-manifest.json`. |
| `artifacts/suite-manifest.json` | Suite execution manifest. | `dry_run=false`; all 46 selected steps succeeded. |
| `artifacts/results.jsonl` | Raw structured result rows. | Recall, latency, storage, build timing, and kernel counter rows. |
| `artifacts/results-report.jsonl` | Report output generated from `results.jsonl`. | Flattened summary rows used by `summary-cold-start-50k.md`. |
| `artifacts/suite-status.log` | Suite status output. | `completed=46 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`. |
| `artifacts/suite-report.log` | Human-readable suite report. | Succeeded; mirrors raw results. |
| `artifacts/suite/*.log` | Per-step load/recall/storage/latency/evict logs. | Load/recall/storage/latency logs contain command output. Per-step evict logs are 0-byte and are not cited for bytes. |
| `artifacts/evict-dry-run-source-f32-w32.log` | Post-run dry-run relation/file coverage for source/f32. | 5 relations, 9 files, 848027648 bytes. |
| `artifacts/evict-dry-run-index-f16-w32.log` | Post-run dry-run relation/file coverage for index/f16. | 5 relations, 9 files, 1014423552 bytes. |
| `artifacts/evict-dry-run-index-rabitq4-w128.log` | Post-run dry-run relation/file coverage for index/rabitq4. | 5 relations, 9 files, 890134528 bytes. |
| `artifacts/evict-dry-run-index-rabitq8-c4-w64.log` | Post-run dry-run relation/file coverage for index/rabitq8 clip4. | 5 relations, 9 files, 931487744 bytes. |
| `artifacts/evict-dry-run-index-turboquant-w32.log` | Post-run dry-run relation/file coverage for index/turboquant. | 5 relations, 9 files, 898826240 bytes. |
| `artifacts/summary-cold-start-50k.md` | Curated packet summary. | Compares recall, NDCG, single-query cold latency, storage, and eviction coverage. |

Generated but intentionally not committed:

- `artifacts/suite/truth-50k-k10.json` is a regenerable ground-truth cache.

## Key Result Lines

Environment:

- `artifacts/suite/precheck-host.log`: PostgreSQL 18.3 on x86_64, `shared_buffers=128MB`, `work_mem=4MB`, `maintenance_work_mem=64MB`, `effective_cache_size=4GB`.

Suite status:

- `artifacts/suite-audit.log`: audit passed, 46 steps.
- `artifacts/suite-status.log`: `completed=46 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

Recall@10:

| Candidate | nprobe 32 | nprobe 128 | nprobe 200 |
| --- | --- | --- | --- |
| source/f32 width32 | 0.9520 | 0.9875 | 0.9895 |
| index/f16 width32 | 0.9520 | 0.9875 | 0.9895 |
| index/rabitq4 width128 | 0.9200 | 0.9450 | 0.9460 |
| index/rabitq8 clip4 width64 | 0.9550 | 0.9915 | 0.9930 |
| index/turboquant width32 | 0.9300 | 0.9550 | 0.9565 |

Single-query cold latency:

| Candidate | nprobe 32 | nprobe 128 | nprobe 200 |
| --- | --- | --- | --- |
| source/f32 width32 | 5.91 ms | 9.99 ms | 12.1 ms |
| index/f16 width32 | 21.7 ms | 10.1 ms | 12.5 ms |
| index/rabitq4 width128 | 17.5 ms | 12.8 ms | 13.7 ms |
| index/rabitq8 clip4 width64 | 7.84 ms | 14.1 ms | 13.3 ms |
| index/turboquant width32 | 7.74 ms | 10.9 ms | 12.7 ms |

Storage:

| Candidate | IVF index | Total relation footprint |
| --- | --- | --- |
| source/f32 width32 | 13.8 MiB | 808.7 MiB |
| index/f16 width32 | 172.5 MiB | 967.4 MiB |
| index/rabitq4 width128 | 54.0 MiB | 848.9 MiB |
| index/rabitq8 clip4 width64 | 93.4 MiB | 888.3 MiB |
| index/turboquant width32 | 62.3 MiB | 857.1 MiB |
