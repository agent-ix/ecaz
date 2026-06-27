# Task 111 Packet 006 Artifact Manifest

- Head SHA: `505f23fb0edbdb8874b7f3e4e7ec7b95c9fe63c0`
- Branch: `task-111-ivf-dense-posting-block-layout`
- Task bucket: `reviews/task-111/006-benchmark-gate`
- Timestamp: `2026-06-16T20:55:29-07:00`
- Benchmark database: `task111_dense_bench_clean`
- Connection: host/socket `/home/peter/.pgrx`, port `28818`, user `peter`
- Backend: release install at `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- Backend sha256: `0aac8902b4e474302500d5d4b6ea119cc874eef53b1b5a7d5094ead98ab84817`
- Surface isolation: yes. The suite uses one table/index prefix per storage-format/layout cell.
- Corpus availability: local real 100k corpus was available; no local 1M manifest was found under `data/`.
- Corpus files: `data/task106_full_sweep_100k/ec_real_100k_corpus.tsv`, `data/task106_full_sweep_100k/ec_real_100k_queries.tsv`, manifest `data/task106_full_sweep_100k/ec_real_100k_manifest.json`.
- Corpus sha256: corpus `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`, queries `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Suite config: `artifacts/task111-dense-posting-suite.json`
- Suite config sha256: `8bcf30ddac6c552fb9af4c47660bd8683205bac11c45c8a0681a25253b2fdc09`
- Suite manifest: `artifacts/suite/suite-manifest.json`
- Parsed report: `artifacts/suite/results-report.jsonl`
- Raw structured results: `artifacts/suite/results.jsonl`
- Recall truth cache was generated during the run but intentionally not committed per repository review-packet policy.

## Commands

```bash
cargo build --release -p ecaz-cli --bin ecaz
target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-111/006-benchmark-gate/artifacts/install-ecaz-pg18-release.log
target/release/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --file <create clean DB SQL> --log-output reviews/task-111/006-benchmark-gate/artifacts/create-task111-clean-bench-db.log
target/release/ecaz dev sql --pg 18 --db task111_dense_bench_clean --socket-dir /home/peter/.pgrx --port 28818 --raw --file <create extension/profile SQL> --log-output reviews/task-111/006-benchmark-gate/artifacts/create-extension-task111-clean-bench.log
target/release/ecaz bench suite run --config reviews/task-111/006-benchmark-gate/artifacts/task111-dense-posting-suite.json --database task111_dense_bench_clean --host /home/peter/.pgrx --port 28818 --user peter
target/release/ecaz bench suite report --manifest reviews/task-111/006-benchmark-gate/artifacts/suite/suite-manifest.json --results-output reviews/task-111/006-benchmark-gate/artifacts/suite/results-report.jsonl
```

## Suite Shape

- Rows: 100000 corpus rows, 100 query rows sampled for recall/latency.
- `k`: 10.
- `nprobe` sweep: 16, 32 for recall and latency.
- Latency: warm cache, concurrency 1, 100 iterations.
- Defaults: `profile=ec_ivf`, `bits=4`, `seed=42`, `pg=18`, `force_index=true`, `sample_backend_memory=false`.
- Common reloptions: `nlists=64`, `nprobe=32`, `training_sample_rows=10000`.
- TurboQuant row: `dense_posting_blocks=0`, `storage_format=turboquant`.
- TurboQuant dense: `dense_posting_blocks=1`, `storage_format=turboquant`.
- RaBitQ row: `dense_posting_blocks=0`, `storage_format=rabitq`, `quant_bits=1`.
- RaBitQ dense: `dense_posting_blocks=1`, `storage_format=rabitq`, `quant_bits=1`.
- Dense page-format note: the dense tag `0x25` layout changed during this experimental/default-off task. Any packet-002-era local dev indexes must be rebuilt; durable promotion still needs Task 42 format coordination.

## Suite Status

The report found 20 completed steps, 0 failed, 0 skipped, 0 dry-run, 0 missing artifacts, and 0 stale artifacts.

| Step group | Artifacts |
| --- | --- |
| load | `artifacts/suite/load-tq-row-real100k.log`, `load-tq-dense-real100k.log`, `load-rb1-row-real100k.log`, `load-rb1-dense-real100k.log` |
| recall | `artifacts/suite/recall-tq-row-real100k.log`, `recall-tq-dense-real100k.log`, `recall-rb1-row-real100k.log`, `recall-rb1-dense-real100k.log` |
| latency | `artifacts/suite/latency-tq-row-real100k.log`, `latency-tq-dense-real100k.log`, `latency-rb1-row-real100k.log`, `latency-rb1-dense-real100k.log` |
| storage | `artifacts/suite/storage-tq-row-real100k.log`, `storage-tq-dense-real100k.log`, `storage-rb1-row-real100k.log`, `storage-rb1-dense-real100k.log` |
| explain | `artifacts/suite/explain-tq-row-real100k.{sql,log}`, `explain-tq-dense-real100k.{sql,log}`, `explain-rb1-row-real100k.{sql,log}`, `explain-rb1-dense-real100k.{sql,log}` |

## Key Results

Build index timing:

| Cell | Build index seconds |
| --- | ---: |
| TurboQuant row | 7.05 |
| TurboQuant dense | 7.38 |
| RaBitQ row | 6.81 |
| RaBitQ dense | 6.77 |

Recall and NDCG:

| Cell | nprobe | recall@10 | NDCG@10 |
| --- | ---: | ---: | ---: |
| TurboQuant row | 16 | 0.8980 | 0.9915 |
| TurboQuant dense | 16 | 0.8980 | 0.9915 |
| TurboQuant row | 32 | 0.9370 | 0.9966 |
| TurboQuant dense | 32 | 0.9370 | 0.9966 |
| RaBitQ row | 16 | 0.7490 | 0.9826 |
| RaBitQ dense | 16 | 0.7490 | 0.9826 |
| RaBitQ row | 32 | 0.7630 | 0.9875 |
| RaBitQ dense | 32 | 0.7630 | 0.9875 |

Warm latency:

| Cell | nprobe | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| TurboQuant row | 16 | 16.5 ms | 20.3 ms | 26.3 ms |
| TurboQuant dense | 16 | 19.9 ms | 23.7 ms | 25.4 ms |
| TurboQuant row | 32 | 31.7 ms | 42.3 ms | 52.0 ms |
| TurboQuant dense | 32 | 39.2 ms | 45.7 ms | 48.2 ms |
| RaBitQ row | 16 | 7.68 ms | 10.1 ms | 11.4 ms |
| RaBitQ dense | 16 | 6.65 ms | 8.17 ms | 9.55 ms |
| RaBitQ row | 32 | 14.4 ms | 16.5 ms | 19.6 ms |
| RaBitQ dense | 32 | 12.3 ms | 13.9 ms | 14.1 ms |

Index storage:

| Cell | Index size | Per row |
| --- | ---: | ---: |
| TurboQuant row | 87.6 MiB | 918.2 B |
| TurboQuant dense | 78.9 MiB | 827.1 B |
| RaBitQ row | 29.7 MiB | 311.3 B |
| RaBitQ dense | 22.5 MiB | 235.8 B |

EXPLAIN scan counters at `nprobe=32`:

| Cell | Posting pages | Postings | Row postings | Dense blocks | Dense postings | Scratch flushes | Scratch payload bytes | Scratch heap TID bytes | Approx scan us |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| TurboQuant row | 4700 | 42171 | 42171 | 0 | 0 | 165 | 32387328 | 253026 | 53387 |
| TurboQuant dense | 4233 | 42171 | 0 | 4233 | 42171 | 0 | 0 | 0 | 45203 |
| RaBitQ row | 1577 | 42171 | 42171 | 0 | 0 | 165 | 8602884 | 253026 | 13958 |
| RaBitQ dense | 1189 | 42171 | 0 | 1189 | 42171 | 0 | 0 | 0 | 10673 |

Batch scorer flush counters from latency logs:

| Cell | nprobe | SIMD flushes | candidates | elapsed ms | width_lt8 | width_8_15 | width_16_31 | width_ge32 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| TurboQuant row | 16 | 10199 | 2599845 | 643.814181 | 4 | 2 | 4 | 10189 |
| TurboQuant dense | 16 | 260725 | 2599779 | 1259.420194 | 1288 | 259437 | 0 | 0 |
| TurboQuant row | 32 | 20379 | 5203807 | 1247.250896 | 4 | 5 | 7 | 20363 |
| TurboQuant dense | 32 | 521755 | 5203613 | 2635.417045 | 2405 | 519350 | 0 | 0 |
| RaBitQ row | 16 | 10199 | 2599851 | 194.824461 | 4 | 2 | 4 | 10189 |
| RaBitQ dense | 16 | 73084 | 2599851 | 231.704573 | 406 | 408 | 639 | 71631 |
| RaBitQ row | 32 | 20380 | 5203809 | 375.964409 | 5 | 5 | 7 | 20363 |
| RaBitQ dense | 32 | 146299 | 5203809 | 472.327454 | 810 | 837 | 1266 | 143386 |

## Interpretation

Dense blocks preserve recall and eliminate row-posting scratch copies for the explain sample. They also reduce index pages/bytes for both storage formats. The promotion gate is still not met as a broad default: TurboQuant dense regressed p50 and p95 at both nprobe cells, and the batch-counter evidence points to many small scorer flushes as the likely cause. RaBitQ improved p50/p95/p99 in this run, but promotion criteria require the active TurboQuant and RaBitQ surfaces to pass without a tail regression that erases the win. Recommendation: iterate, keep the gate off by default, and focus the next slice on dense-block packing / scan coalescing before any promotion decision.
