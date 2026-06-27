# Task 111g Packet 005 Artifact Manifest

- head SHA: `a7cdb86fe021fa11db0ea00ac07c47c8896d7f1a`
- task bucket: `reviews/task-111g/`
- packet path: `reviews/task-111g/005-direct-sidecar-rerank-tids/`
- timestamp: `2026-06-19T16:14:39-07:00`
- lane: local PG18 IVF sidecar-index rerank attribution
- database: `task111g_direct_tid`
- socket/port: `/home/peter/.pgrx`, `28818`
- run surface: isolated one-index-per-table prefixes from the suite (`attr_idx_{f16,rq4}_{10k,50k,100k}`)
- storage formats: `storage_format=coarse_rerank`, `rerank_placement=index`, `rerank_width=64`
- rerank formats: `f16`, `rabitq4`
- suite source config used by the run: `benchmarks/ivf-111g-115-attribution/configs/sidecar-index-placement.json`
- packet-local suite config copy: `artifacts/sidecar-index-direct-tids/suite-config.json`
- prior comparison packet: `benchmarks/ivf-111g-115-attribution/artifacts/adr079-sidecar-index/results.jsonl`

## Commands

Validation:

```sh
script -q -e -c "cargo test --no-default-features --features pg18 posting_scratch_soa" reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/cargo-test-posting-scratch-soa.log
script -q -e -c "cargo check --no-default-features --features pg18" reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/cargo-check-pg18.log
```

Install and database setup:

```sh
script -q -e -c "cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18" reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/cargo-pgrx-install-pg18-release.log
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 dev sql --raw --sql "SELECT version();" --log-output reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/pg18-connectivity.log
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 dev sql --raw --sql "DROP DATABASE IF EXISTS task111g_direct_tid;" --log-output reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/drop-benchmark-db.log
target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 dev sql --raw --sql "CREATE DATABASE task111g_direct_tid;" --log-output reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/create-benchmark-db.log
target/debug/ecaz --database task111g_direct_tid --host /home/peter/.pgrx --port 28818 dev sql --raw --sql "CREATE EXTENSION IF NOT EXISTS ecaz; SELECT ecaz_build_profile();" --log-output reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/create-extension.log
```

Suite dry-run, run, and status:

```sh
target/debug/ecaz --database task111g_direct_tid --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/suite-dry-run.log bench suite run --config benchmarks/ivf-111g-115-attribution/configs/sidecar-index-placement.json --artifact-dir reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids --manifest-output reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/suite-manifest.dry-run.json --results-output reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/results.dry-run.jsonl --dry-run
target/debug/ecaz --database task111g_direct_tid --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/suite-run.log bench suite run --config benchmarks/ivf-111g-115-attribution/configs/sidecar-index-placement.json --artifact-dir reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids --manifest-output reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/suite-manifest.json --results-output reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/results.jsonl
target/debug/ecaz --log-file reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/suite-status.log bench suite status --manifest reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/suite-manifest.json
```

## Artifact Index

- `cargo-test-posting-scratch-soa.log`: focused unit validation. Key line: `5 passed; 0 failed`.
- `cargo-check-pg18.log`: PG18 static validation. Key line: `Finished dev profile`.
- `cargo-pgrx-install-pg18-release.log`: release PG18 extension install. Key lines: copied `ecaz.so`; `Finished installing ecaz`.
- `pg18-connectivity.log`: PG18 server version probe.
- `drop-benchmark-db.log`, `create-benchmark-db.log`: dedicated database reset.
- `create-extension.log`: `CREATE EXTENSION`; `ecaz_build_profile = release`.
- `suite-dry-run.log`: expanded suite command list.
- `suite-run.log`: full suite transcript.
- `suite-status.log`: status summary. Key line: `completed=24 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `sidecar-index-direct-tids/suite-config.json`: packet-local copy of the suite config.
- `sidecar-index-direct-tids/suite-manifest.dry-run.json`: dry-run manifest.
- `sidecar-index-direct-tids/suite-manifest.json`: completed suite manifest.
- `sidecar-index-direct-tids/results.jsonl`: normalized suite results.
- `sidecar-index-direct-tids/load-*.log`: load/build logs for each format and scale.
- `sidecar-index-direct-tids/recall-*.log`: recall logs for each format and scale.
- `sidecar-index-direct-tids/latency-*.log`: latency logs for each format and scale.
- `sidecar-index-direct-tids/storage-*.log`: storage logs for each format and scale.

## Key Result Lines

All latency rows are p50, `post_recall_warm`, `iterations=200`, `concurrency=1`.

| cell | nprobe 8 | nprobe 64 | nprobe 200 |
| --- | ---: | ---: | ---: |
| f16 index 10k, old ADR-079 packet | 4.81 ms | 6.03 ms | 6.44 ms |
| f16 index 10k, direct TID packet | 2.10 ms | 3.00 ms | 3.47 ms |
| f16 index 50k, old ADR-079 packet | 78.3 ms | 80.6 ms | 91.2 ms |
| f16 index 50k, direct TID packet | 3.79 ms | 4.62 ms | 8.95 ms |
| f16 index 100k, old ADR-079 packet | 146.8 ms | 150.2 ms | 159.2 ms |
| f16 index 100k, direct TID packet | 2.99 ms | 6.02 ms | 13.0 ms |
| rabitq4 index 100k, old ADR-079 packet | 7.67 ms | 9.60 ms | 16.0 ms |
| rabitq4 index 100k, direct TID packet | 2.79 ms | 5.72 ms | 11.9 ms |

Recall stayed in the expected sweep shape. At 100k, f16 recall@10 was
`0.7670/0.8520/0.9225/0.9640/0.9860/0.9975` for nprobe
`8/16/32/64/128/200`; rabitq4 recall@10 was
`0.7465/0.8235/0.8840/0.9165/0.9345/0.9420`.

Storage rows match the prior sidecar-index packet at the same scales:
f16 index size is `42.5 MiB`, `209.1 MiB`, `416.6 MiB`; rabitq4 index size is
`11.2 MiB`, `52.6 MiB`, `103.6 MiB`.
