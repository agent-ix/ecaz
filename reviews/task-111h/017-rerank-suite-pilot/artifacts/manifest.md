# Task 111h Packet 017 Artifact Manifest

- head SHA: `2b1a6d1b3bea7ae2e5ba856bb5bda185f75c6a02`
- task bucket: `reviews/task-111h/017-rerank-suite-pilot`
- captured at: `2026-06-20T07:39:07Z`
- lane: local PG18, `/home/peter/.pgrx`, port `28818`
- database: `task111h_rerank_pilot`
- backend: `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- backend sha256: `1405638e2298eb7465b44b4204c33e78f65f2121a7963f69d712bb1329ed30be`
- corpus prefix: `data/staged-current/ec_real_10k`
- corpus sha256: `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`
- query sha256: `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- surface isolation: one prefix/table/index per rerank format and width cell

## Suite Shape

Config: `artifacts/task111h-10k-rerank-format-width-suite.json`

Matrix:

- corpus scale: 10k rows, 200 queries, dim 1536
- formats: source-side f32, index f16, index rabitq4, index rabitq8, index turboquant
- rerank widths: 32, 64, 128, 256
- load reloptions: `nlists=256`, `nprobe=32`, `training_sample_rows=10000`, `storage_format=coarse_rerank`, `coarse_bits=1`, `rerank=heap_f32`
- recall/latency sweep: nprobe `8,16,32,64,128,200`
- latency: 200 iterations, concurrency 1, force-index, post-recall-warm, memory samples, Task 87 candidate/batch counters

This is a 10k pilot matrix only. It is not the full 111h acceptance sweep for larger corpora, cold/cache-state variants, table-owned storage, or legacy 0x2A vanilla IVF comparison.

## Commands

Audit:

```sh
target/debug/ecaz bench suite audit --config reviews/task-111h/017-rerank-suite-pilot/artifacts/task111h-10k-rerank-format-width-suite.json --database task111h_rerank_pilot --host /home/peter/.pgrx --log-file reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-audit.log
```

Dry run:

```sh
target/debug/ecaz bench suite run --dry-run --config reviews/task-111h/017-rerank-suite-pilot/artifacts/task111h-10k-rerank-format-width-suite.json --artifact-dir reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-dry-run --database task111h_rerank_pilot --host /home/peter/.pgrx --manifest-output reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-dry-run-manifest.json --log-file reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-dry-run.log
```

Build:

```sh
script -q -c "cargo build --release --no-default-features --features pg18" reviews/task-111h/017-rerank-suite-pilot/artifacts/cargo-build-release-pg18.log
```

Install:

```sh
target/release/ecaz dev install ecaz-pg-test --pg 18 --pgrx-home /home/peter/.pgrx --log-file reviews/task-111h/017-rerank-suite-pilot/artifacts/install-ecaz-pg18-release.log
```

Database setup:

```sh
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "SELECT datname FROM pg_database WHERE datname = 'task111h_rerank_pilot';" --log-output reviews/task-111h/017-rerank-suite-pilot/artifacts/db-exists.log
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "CREATE DATABASE task111h_rerank_pilot;" --log-output reviews/task-111h/017-rerank-suite-pilot/artifacts/create-db.log
target/release/ecaz dev sql --pg 18 --db task111h_rerank_pilot --socket-dir /home/peter/.pgrx --raw --sql "CREATE EXTENSION ecaz;" --log-output reviews/task-111h/017-rerank-suite-pilot/artifacts/create-extension.log
```

Suite run:

```sh
target/release/ecaz bench suite run --config reviews/task-111h/017-rerank-suite-pilot/artifacts/task111h-10k-rerank-format-width-suite.json --artifact-dir reviews/task-111h/017-rerank-suite-pilot/artifacts/suite --database task111h_rerank_pilot --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-manifest.json --results-output reviews/task-111h/017-rerank-suite-pilot/artifacts/results.jsonl --log-file reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-run.log
```

Status and report:

```sh
target/release/ecaz bench suite status --manifest reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-manifest.json --database task111h_rerank_pilot --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-status.log
target/release/ecaz bench suite report --manifest reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-manifest.json --results-output reviews/task-111h/017-rerank-suite-pilot/artifacts/results-report.jsonl --database task111h_rerank_pilot --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/017-rerank-suite-pilot/artifacts/suite-report.log
```

The suite generated `artifacts/suite/truth-10k-k10.json` as a runtime ground-truth cache. It is intentionally not listed as durable evidence and should not be committed; the corpus/query SHAs above and recall logs/results are the durable evidence.

## Artifact Index

- `artifacts/task111h-10k-rerank-format-width-suite.json`: suite config.
- `artifacts/suite-audit.log`: audit output.
- `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json`: dry-run output.
- `artifacts/cargo-build-release-pg18.log`: release PG18 build output.
- `artifacts/install-ecaz-pg18-release.log`: PG18 install output.
- `artifacts/db-exists.log`, `artifacts/create-db.log`, `artifacts/create-extension.log`: database setup logs.
- `artifacts/suite-run.log`: suite run driver log.
- `artifacts/suite-manifest.json`: suite manifest enumerating every step and packet-local step log.
- `artifacts/results.jsonl`: suite structured results emitted by the run.
- `artifacts/suite-status.log`: suite status output.
- `artifacts/suite-report.log`: suite report output.
- `artifacts/results-report.jsonl`: structured report results.
- `artifacts/summary-nprobe32.md`: derived nprobe=32 comparison table from `results-report.jsonl`.
- `artifacts/suite/*.log`: per-step load, recall, latency, storage, and precheck logs; enumerated in `suite-manifest.json`.

## Key Result Lines

Suite completion:

```text
[suite:task111h-10k-rerank-format-width-pilot] completed=81 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
- steps: completed 81, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0
```

At nprobe=32:

```text
placement	format	width	recall32	latency32_p50	index_size	index_per_row_b	total_size
source	f32	32	0.9970	2.98 ms	5.1 MiB	538.2	164.2 MiB
source	f32	64	0.9985	3.64 ms	5.1 MiB	538.2	164.2 MiB
source	f32	128	0.9985	5.69 ms	5.1 MiB	538.2	164.2 MiB
source	f32	256	0.9985	9.27 ms	5.1 MiB	538.2	164.2 MiB
index	f16	32	0.9960	2.20 ms	37.0 MiB	3875.6	196.0 MiB
index	f16	64	0.9975	2.31 ms	36.0 MiB	3771.6	195.0 MiB
index	f16	128	0.9975	2.79 ms	35.8 MiB	3752.8	194.8 MiB
index	f16	256	0.9975	4.17 ms	35.8 MiB	3751.1	194.8 MiB
index	rabitq4	32	0.9775	1.73 ms	14.7 MiB	1544.2	173.8 MiB
index	rabitq4	64	0.9775	1.85 ms	13.9 MiB	1462.3	173.0 MiB
index	rabitq4	128	0.9775	2.18 ms	13.8 MiB	1445.9	172.8 MiB
index	rabitq4	256	0.9775	2.29 ms	13.8 MiB	1444.2	172.8 MiB
index	rabitq8	32	0.9845	2.75 ms	22.3 MiB	2334.7	181.3 MiB
index	rabitq8	64	0.9850	1.91 ms	21.3 MiB	2238.1	180.4 MiB
index	rabitq8	128	0.9850	2.75 ms	21.2 MiB	2219.2	180.2 MiB
index	rabitq8	256	0.9850	2.66 ms	21.1 MiB	2217.6	180.2 MiB
index	turboquant	32	0.9730	2.09 ms	14.7 MiB	1540.1	173.7 MiB
index	turboquant	64	0.9730	2.28 ms	13.9 MiB	1458.2	172.9 MiB
index	turboquant	128	0.9730	2.43 ms	13.7 MiB	1439.3	172.8 MiB
index	turboquant	256	0.9730	2.85 ms	13.7 MiB	1437.7	172.7 MiB
```

Latency range across all 120 latency rows:

```text
p50 min: task111h017_10k_index_rabitq4_w32 nprobe=8 p50=1.59 ms max=4.11 ms
p50 max: task111h017_10k_source_f32_w256 nprobe=200 p50=10.4 ms max=15.8 ms
```
