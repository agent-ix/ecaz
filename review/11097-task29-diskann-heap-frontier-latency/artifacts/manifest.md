# Artifact Manifest

Head SHA: `6bd9c5ec42bdd5dcc182c1b3d8efcac72b1819d5`

Packet: `review/11097-task29-diskann-heap-frontier-latency`

Lane: Task 29 DiskANN initial tuning, persisted scan latency.

Fixture: `task29_diskann_real10k`, local PG18, real-10k 1536-d corpus,
200 query rows.

Storage format: `ec_diskann` `pq_fastscan` tuple format with binary-sidecar
prefilter from Task 29a.

Rerank mode: heap-f32 exact rerank, existing reloption default
`rerank_budget=64`.

Table model: isolated one-index-per-table prefix
`task29_diskann_real10k`.

Cache state: warm local run. The benchmark is intended as an apples-to-apples
comparison with `review/11096` on the same local database, not as a cold-cache
or product benchmark.

## Artifacts

### `recall-sidecar-heap-frontier-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11097-task29-diskann-heap-frontier-latency/artifacts/recall-sidecar-heap-frontier-cli.log bench recall --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --force-index --truth-cache-file review/11088-task29-diskann-seeded-build-probe/artifacts/real10k-truth-k10.json --log-output review/11097-task29-diskann-heap-frontier-latency/artifacts/recall-sidecar-heap-frontier-table.log`

Timestamp: 2026-04-30 local.

Key result rows:

- L=64: recall@10 `0.9955`, NDCG `0.9997`, mean `52.95 ms`
- L=128: recall@10 `0.9960`, NDCG `0.9999`, mean `55.86 ms`
- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean `64.81 ms`
- L=400: recall@10 `0.9970`, NDCG `0.9999`, mean `79.49 ms`
- L=800: recall@10 `0.9975`, NDCG `0.9999`, mean `108.25 ms`

### `latency-sidecar-heap-frontier-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11097-task29-diskann-heap-frontier-latency/artifacts/latency-sidecar-heap-frontier-cli.log bench latency --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --iterations 200 --concurrency 1 --force-index --sample-backend-memory --log-output review/11097-task29-diskann-heap-frontier-latency/artifacts/latency-sidecar-heap-frontier-table.log`

Timestamp: 2026-04-30 local.

Key result rows:

- L=64: mean `51.5 ms`, p50 `50.9 ms`, p95 `61.7 ms`, p99 `71.0 ms`, HWM `63740 KiB`
- L=128: mean `56.8 ms`, p50 `57.2 ms`, p95 `61.9 ms`, p99 `63.6 ms`, HWM `65020 KiB`
- L=200: mean `62.9 ms`, p50 `62.7 ms`, p95 `70.4 ms`, p99 `73.4 ms`, HWM `64860 KiB`
- L=400: mean `78.1 ms`, p50 `80.1 ms`, p95 `92.3 ms`, p99 `96.6 ms`, HWM `65340 KiB`
- L=800: mean `109.8 ms`, p50 `110.8 ms`, p95 `137.9 ms`, p99 `149.1 ms`, HWM `65596 KiB`
