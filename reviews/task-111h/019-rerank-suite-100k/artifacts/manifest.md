# Task 111h Packet 019 Artifact Manifest

- head SHA: `3cf84fd845e5ee5486cd12b5c9f805ca1d893a1f`
- branch: `bench-ivf-111g-115-attribution`
- task bucket: `reviews/task-111h/019-rerank-suite-100k`
- captured at: `2026-06-20T09:28:49Z`
- lane: local PG18, `/home/peter/.pgrx`, port `28818`
- database: `task111h_rerank_100k`
- backend: `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- backend sha256: `1405638e2298eb7465b44b4204c33e78f65f2121a7963f69d712bb1329ed30be`
- corpus prefix: `data/staged-current/ec_real_100k`
- corpus sha256: `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`
- query sha256: `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
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

The installed backend hash matches Packets 017 and 018.

## Suite Shape

Config: `artifacts/task111h-100k-rerank-format-width-suite.json`

Matrix:

- corpus scale: 100k rows, 200 query limit per recall step, dim 1536
- formats: source-side f32, index f16, index rabitq4, index rabitq8, index turboquant
- rerank widths: 32, 64, 128, 256
- load reloptions: `nlists=256`, `nprobe=32`, `training_sample_rows=10000`, `storage_format=coarse_rerank`, `coarse_bits=1`, `rerank=heap_f32`
- recall/latency sweep: nprobe `8,16,32,64,128,200`
- latency: 200 iterations, concurrency 1, force-index, post-recall-warm, memory samples, Task 87 candidate/batch counters

This is a 100k warm-cache local matrix. It is not the full 111h acceptance
sweep for 1M corpus scale, cold/cache-state variants, table-owned compact
storage, remote hosts, or the legacy 0x2A vanilla IVF comparison.

## Commands

Audit:

```sh
target/release/ecaz bench suite audit --config reviews/task-111h/019-rerank-suite-100k/artifacts/task111h-100k-rerank-format-width-suite.json --database task111h_rerank_100k --host /home/peter/.pgrx --log-file reviews/task-111h/019-rerank-suite-100k/artifacts/suite-audit.log
```

Dry run:

```sh
target/release/ecaz bench suite run --dry-run --config reviews/task-111h/019-rerank-suite-100k/artifacts/task111h-100k-rerank-format-width-suite.json --artifact-dir reviews/task-111h/019-rerank-suite-100k/artifacts/suite-dry-run --database task111h_rerank_100k --host /home/peter/.pgrx --manifest-output reviews/task-111h/019-rerank-suite-100k/artifacts/suite-dry-run-manifest.json --log-file reviews/task-111h/019-rerank-suite-100k/artifacts/suite-dry-run.log
```

Build:

```sh
script -q -c "cargo build --release --no-default-features --features pg18" reviews/task-111h/019-rerank-suite-100k/artifacts/cargo-build-release-pg18.log
```

Install:

```sh
target/release/ecaz dev install ecaz-pg-test --pg 18 --pgrx-home /home/peter/.pgrx --log-file reviews/task-111h/019-rerank-suite-100k/artifacts/install-ecaz-pg18-release.log
```

Database setup:

```sh
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "SELECT datname FROM pg_database WHERE datname = 'task111h_rerank_100k';" --log-output reviews/task-111h/019-rerank-suite-100k/artifacts/db-exists.log
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --sql "CREATE DATABASE task111h_rerank_100k;" --log-output reviews/task-111h/019-rerank-suite-100k/artifacts/create-db.log
target/release/ecaz dev sql --pg 18 --db task111h_rerank_100k --socket-dir /home/peter/.pgrx --raw --sql "CREATE EXTENSION ecaz;" --log-output reviews/task-111h/019-rerank-suite-100k/artifacts/create-extension.log
```

Suite run:

```sh
target/release/ecaz bench suite run --config reviews/task-111h/019-rerank-suite-100k/artifacts/task111h-100k-rerank-format-width-suite.json --artifact-dir reviews/task-111h/019-rerank-suite-100k/artifacts/suite --database task111h_rerank_100k --host /home/peter/.pgrx --port 28818 --continue-on-error --manifest-output reviews/task-111h/019-rerank-suite-100k/artifacts/suite-manifest.json --results-output reviews/task-111h/019-rerank-suite-100k/artifacts/results.jsonl --log-file reviews/task-111h/019-rerank-suite-100k/artifacts/suite-run.log
```

Status and report:

```sh
target/release/ecaz bench suite status --manifest reviews/task-111h/019-rerank-suite-100k/artifacts/suite-manifest.json --database task111h_rerank_100k --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/019-rerank-suite-100k/artifacts/suite-status.log
target/release/ecaz bench suite report --manifest reviews/task-111h/019-rerank-suite-100k/artifacts/suite-manifest.json --results-output reviews/task-111h/019-rerank-suite-100k/artifacts/results-report.jsonl --database task111h_rerank_100k --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/019-rerank-suite-100k/artifacts/suite-report.log
```

The suite generated `artifacts/suite/truth-100k-k10.json` as a runtime
ground-truth cache. It is intentionally not listed as durable evidence and was
removed before commit; the corpus/query SHAs above and recall logs/results are
the durable evidence.

## Artifact Index

- `artifacts/task111h-100k-rerank-format-width-suite.json`: suite config.
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
[suite:task111h-100k-rerank-format-width] completed=81 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
- steps: completed 81, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0
```

At nprobe=32:

```text
placement	format	width	recall@10	ndcg@10	p50 latency	p95 latency	index size	index B/row	total size	build_s
source	f32	32	0.9285	0.9947	4.57 ms	5.13 ms	24.6 MiB	258.2	1.6 GiB	9.230000
source	f32	64	0.9350	0.9948	5.35 ms	6.09 ms	24.6 MiB	258.2	1.6 GiB	8.980000
source	f32	128	0.9350	0.9948	7.41 ms	8.72 ms	24.6 MiB	258.2	1.6 GiB	8.830000
source	f32	256	0.9350	0.9948	12.2 ms	15.4 ms	24.6 MiB	258.2	1.6 GiB	9.130000
index	f16	32	0.9280	0.9947	4.38 ms	5.52 ms	342.0 MiB	3586.5	1.9 GiB	12.730000
index	f16	64	0.9345	0.9948	5.92 ms	7.66 ms	330.1 MiB	3461.8	1.9 GiB	12.960000
index	f16	128	0.9345	0.9948	8.98 ms	13.1 ms	324.3 MiB	3400.3	1.9 GiB	12.940000
index	f16	256	0.9345	0.9948	13.8 ms	20.0 ms	323.7 MiB	3394.4	1.9 GiB	12.850000
index	rabitq4	32	0.8895	0.9942	3.76 ms	4.34 ms	121.8 MiB	1277.5	1.7 GiB	11.530000
index	rabitq4	64	0.8930	0.9943	4.30 ms	4.97 ms	110.2 MiB	1155.2	1.7 GiB	11.760000
index	rabitq4	128	0.8930	0.9943	6.82 ms	8.11 ms	104.5 MiB	1095.5	1.7 GiB	11.550000
index	rabitq4	256	0.8930	0.9943	6.19 ms	8.38 ms	104.0 MiB	1090.8	1.7 GiB	10.970000
index	rabitq8	32	0.8990	0.9944	4.21 ms	5.23 ms	195.4 MiB	2049.0	1.7 GiB	11.730000
index	rabitq8	64	0.9000	0.9945	4.62 ms	5.89 ms	183.6 MiB	1925.4	1.7 GiB	11.720000
index	rabitq8	128	0.9000	0.9945	6.05 ms	7.85 ms	177.9 MiB	1865.1	1.7 GiB	11.970000
index	rabitq8	256	0.9000	0.9945	9.52 ms	12.7 ms	177.4 MiB	1860.2	1.7 GiB	13.250000
index	turboquant	32	0.8965	0.9944	3.83 ms	4.35 ms	121.8 MiB	1276.8	1.7 GiB	10.930000
index	turboquant	64	0.9005	0.9945	4.18 ms	4.83 ms	110.1 MiB	1154.4	1.7 GiB	11.670000
index	turboquant	128	0.9000	0.9945	5.00 ms	5.76 ms	104.4 MiB	1094.3	1.7 GiB	11.570000
index	turboquant	256	0.9000	0.9945	6.10 ms	7.34 ms	101.8 MiB	1067.9	1.7 GiB	10.880000
```

Latency range across all 120 latency rows:

```text
p50 min: task111h019_100k_index_rabitq4_w32 nprobe=8 p50=2.15 ms max=4.82 ms
p50 max: task111h019_100k_index_f16_w256 nprobe=200 p50=32.9 ms max=56.4 ms
max query: task111h019_100k_index_f16_w256 nprobe=32 p50=13.8 ms max=211.8 ms
```

Turboquant nprobe32 block-kernel counters:

```text
task111h019_100k_index_turboquant_w32	6400 candidates	1.653836 ms	200 flushes
task111h019_100k_index_turboquant_w64	12800 candidates	3.257094 ms	200 flushes
task111h019_100k_index_turboquant_w128	25600 candidates	6.829726 ms	200 flushes
task111h019_100k_index_turboquant_w256	51200 candidates	12.567503 ms	200 flushes
```
