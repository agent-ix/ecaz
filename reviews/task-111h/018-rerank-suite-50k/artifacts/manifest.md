# Task 111h Packet 018 Artifact Manifest

- head SHA: `30df185a6b0109f0d1cca64a9978b56fe77772d7`
- task bucket: `reviews/task-111h/018-rerank-suite-50k`
- captured at: `2026-06-20T08:26:25Z`
- lane: local PG18, `/home/peter/.pgrx`, port `28818`
- database: `task111h_rerank_50k`
- backend: `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- backend sha256: `1405638e2298eb7465b44b4204c33e78f65f2121a7963f69d712bb1329ed30be`
- corpus prefix: `data/staged-current/ec_real_50k`
- corpus sha256: `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133`
- query sha256: `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`
- surface isolation: one prefix/table/index per rerank format and width cell

## Provenance Caveat

This packet is measurement evidence only and does not stage code changes.
The release build and suite ran with unrelated local formatting-only changes
present in the worktree:

- `crates/ecaz-cli/src/commands/bench/comparator.rs`
- `crates/ecaz-cli/src/commands/bench/suite.rs`
- `crates/ecaz-cli/src/commands/corpus/subset.rs`
- `src/am/ec_ivf/lazy.rs`
- `src/am/ec_ivf/quantizer.rs`
- `src/quant/int8_approx32/avx2.rs`
- `src/quant/isa.rs`
- `src/quant/lut32/sve.rs`
- `src/quant/qjl32/neon.rs`

The installed backend hash matches Packet 017's backend hash.

## Suite Shape

Config: `artifacts/task111h-50k-rerank-format-width-suite.json`

Matrix:

- corpus scale: 50k rows, 200 query limit per recall step, dim 1536
- formats: source-side f32, index f16, index rabitq4, index rabitq8, index turboquant
- rerank widths: 32, 64, 128, 256
- load reloptions: `nlists=256`, `nprobe=32`, `training_sample_rows=10000`, `storage_format=coarse_rerank`, `coarse_bits=1`, `rerank=heap_f32`
- recall/latency sweep: nprobe `8,16,32,64,128,200`
- latency: 200 iterations, concurrency 1, force-index, post-recall-warm, memory samples, Task 87 candidate/batch counters

This is a 50k warm-cache local matrix. It is not the full 111h acceptance sweep
for 100k/1M corpora, cold/cache-state variants, table-owned storage, remote
hosts, or the legacy 0x2A vanilla IVF comparison.

## Commands

Audit:

```sh
target/release/ecaz bench suite audit --config reviews/task-111h/018-rerank-suite-50k/artifacts/task111h-50k-rerank-format-width-suite.json --database task111h_rerank_50k --host /home/peter/.pgrx --log-file reviews/task-111h/018-rerank-suite-50k/artifacts/suite-audit.log
```

Dry run:

```sh
target/release/ecaz bench suite run --dry-run --config reviews/task-111h/018-rerank-suite-50k/artifacts/task111h-50k-rerank-format-width-suite.json --artifact-dir reviews/task-111h/018-rerank-suite-50k/artifacts/suite-dry-run --database task111h_rerank_50k --host /home/peter/.pgrx --manifest-output reviews/task-111h/018-rerank-suite-50k/artifacts/suite-dry-run-manifest.json --log-file reviews/task-111h/018-rerank-suite-50k/artifacts/suite-dry-run.log
```

Build:

```sh
script -q -c "cargo build --release --no-default-features --features pg18" reviews/task-111h/018-rerank-suite-50k/artifacts/cargo-build-release-pg18.log
```

Install:

```sh
target/release/ecaz dev install ecaz-pg-test --pg 18 --pgrx-home /home/peter/.pgrx --log-file reviews/task-111h/018-rerank-suite-50k/artifacts/install-ecaz-pg18-release.log
```

Database setup:

```sh
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "SELECT datname FROM pg_database WHERE datname = 'task111h_rerank_50k';" --log-output reviews/task-111h/018-rerank-suite-50k/artifacts/db-exists.log
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "CREATE DATABASE task111h_rerank_50k;" --log-output reviews/task-111h/018-rerank-suite-50k/artifacts/create-db.log
target/release/ecaz dev sql --pg 18 --db task111h_rerank_50k --socket-dir /home/peter/.pgrx --raw --sql "CREATE EXTENSION ecaz;" --log-output reviews/task-111h/018-rerank-suite-50k/artifacts/create-extension.log
```

Suite run:

```sh
target/release/ecaz bench suite run --config reviews/task-111h/018-rerank-suite-50k/artifacts/task111h-50k-rerank-format-width-suite.json --artifact-dir reviews/task-111h/018-rerank-suite-50k/artifacts/suite --database task111h_rerank_50k --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/018-rerank-suite-50k/artifacts/suite-manifest.json --results-output reviews/task-111h/018-rerank-suite-50k/artifacts/results.jsonl --log-file reviews/task-111h/018-rerank-suite-50k/artifacts/suite-run.log
```

Status and report:

```sh
target/release/ecaz bench suite status --manifest reviews/task-111h/018-rerank-suite-50k/artifacts/suite-manifest.json --database task111h_rerank_50k --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/018-rerank-suite-50k/artifacts/suite-status.log
target/release/ecaz bench suite report --manifest reviews/task-111h/018-rerank-suite-50k/artifacts/suite-manifest.json --results-output reviews/task-111h/018-rerank-suite-50k/artifacts/results-report.jsonl --database task111h_rerank_50k --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/018-rerank-suite-50k/artifacts/suite-report.log
```

The suite generated `artifacts/suite/truth-50k-k10.json` as a runtime
ground-truth cache. It is intentionally not listed as durable evidence and was
removed before commit; the corpus/query SHAs above and recall logs/results are
the durable evidence.

## Artifact Index

- `artifacts/task111h-50k-rerank-format-width-suite.json`: suite config.
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
- `artifacts/summary-nprobe32.md`: derived nprobe=32 comparison table from `results.jsonl`.
- `artifacts/suite/*.log`: per-step load, recall, latency, storage, and precheck logs; enumerated in `suite-manifest.json`.

## Key Result Lines

Suite completion:

```text
[suite:task111h-50k-rerank-format-width] completed=81 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
- steps: completed 81, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0
```

At nprobe=32:

```text
placement	format	width	recall@10	ndcg@10	p50 latency	p95 latency	index size	index B/row	total size	build_s
source	f32	32	0.9520	0.9973	3.74 ms	4.11 ms	13.8 MiB	290.3	808.7 MiB	5.730000
source	f32	64	0.9590	0.9974	4.50 ms	4.87 ms	13.8 MiB	290.3	808.7 MiB	6.060000
source	f32	128	0.9600	0.9974	6.42 ms	6.92 ms	13.8 MiB	290.3	808.7 MiB	6.040000
source	f32	256	0.9600	0.9974	10.2 ms	11.2 ms	13.8 MiB	290.3	808.7 MiB	5.390000
index	f16	32	0.9520	0.9973	3.08 ms	3.64 ms	172.5 MiB	3618.2	967.4 MiB	7.050000
index	f16	64	0.9590	0.9974	4.31 ms	5.36 ms	166.7 MiB	3495.5	961.5 MiB	7.380000
index	f16	128	0.9600	0.9974	6.80 ms	9.42 ms	164.0 MiB	3439.8	958.9 MiB	7.620000
index	f16	256	0.9600	0.9974	9.31 ms	11.7 ms	163.5 MiB	3429.5	958.4 MiB	7.120000
index	rabitq4	32	0.9180	0.9969	2.57 ms	2.87 ms	62.3 MiB	1307.0	857.2 MiB	6.490000
index	rabitq4	64	0.9200	0.9970	3.04 ms	3.55 ms	56.6 MiB	1187.8	851.5 MiB	6.990000
index	rabitq4	128	0.9200	0.9970	3.38 ms	4.12 ms	54.0 MiB	1132.5	848.9 MiB	6.720000
index	rabitq4	256	0.9200	0.9970	5.30 ms	6.17 ms	53.7 MiB	1125.3	848.5 MiB	6.860000
index	rabitq8	32	0.9230	0.9970	2.73 ms	3.14 ms	99.2 MiB	2080.4	894.1 MiB	7.130000
index	rabitq8	64	0.9265	0.9971	3.42 ms	4.15 ms	93.4 MiB	1959.5	888.3 MiB	7.040000
index	rabitq8	128	0.9260	0.9971	4.15 ms	5.38 ms	90.8 MiB	1903.3	885.6 MiB	7.060000
index	rabitq8	256	0.9260	0.9971	7.21 ms	8.56 ms	90.5 MiB	1896.9	885.3 MiB	7.150000
index	turboquant	32	0.9175	0.9970	2.69 ms	3.00 ms	62.3 MiB	1306.3	857.1 MiB	6.700000
index	turboquant	64	0.9195	0.9971	3.31 ms	4.35 ms	56.6 MiB	1187.0	851.5 MiB	6.560000
index	turboquant	128	0.9200	0.9971	3.47 ms	4.11 ms	53.9 MiB	1129.8	848.7 MiB	6.390000
index	turboquant	256	0.9200	0.9971	4.35 ms	5.17 ms	52.8 MiB	1107.2	847.7 MiB	6.260000
```

Latency range across all 120 latency rows:

```text
p50 min: task111h018_50k_index_rabitq4_w32 nprobe=8 p50=1.83 ms max=4.43 ms
p50 max: task111h018_50k_index_f16_w256 nprobe=128 p50=19.7 ms max=37.2 ms
max query: task111h018_50k_index_f16_w256 nprobe=200 p50=15.9 ms max=38.6 ms
```

Turboquant nprobe32 block-kernel counters:

```text
task111h018_50k_index_turboquant_w32	6400 candidates	1.596921 ms	200 flushes
task111h018_50k_index_turboquant_w64	12800 candidates	3.318962 ms	200 flushes
task111h018_50k_index_turboquant_w128	25600 candidates	5.922408 ms	200 flushes
task111h018_50k_index_turboquant_w256	51200 candidates	12.496158 ms	200 flushes
```
