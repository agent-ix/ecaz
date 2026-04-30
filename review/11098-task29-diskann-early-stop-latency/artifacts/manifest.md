# Artifact Manifest

Head SHA: `27bb6af8a037b29918f13ca894cc1c1a466c834d`

Packet: `review/11098-task29-diskann-early-stop-latency`

Lane: Task 29 DiskANN initial tuning, persisted scan early-stop latency.

Fixture: `task29_diskann_real10k`, local PG18, real-10k 1536-d corpus,
200 query rows.

Storage format: `ec_diskann` `pq_fastscan` tuple format with binary-sidecar
prefilter from Task 29a.

Rerank mode: heap-f32 exact rerank, existing reloption default
`rerank_budget=64`.

Table model: isolated one-index-per-table prefix
`task29_diskann_real10k`.

Cache state: warm local run. This is an apples-to-apples comparison with
`review/11096` and `review/11097` on the same local database.

## Artifacts

### `recall-sidecar-early-stop-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11098-task29-diskann-early-stop-latency/artifacts/recall-sidecar-early-stop-cli.log bench recall --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --force-index --truth-cache-file review/11088-task29-diskann-seeded-build-probe/artifacts/real10k-truth-k10.json --log-output review/11098-task29-diskann-early-stop-latency/artifacts/recall-sidecar-early-stop-table.log`

Timestamp: 2026-04-30 local.

Key result rows:

- L=64: recall@10 `0.9955`, NDCG `0.9997`, mean `50.36 ms`
- L=128: recall@10 `0.9960`, NDCG `0.9999`, mean `48.80 ms`
- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean `53.15 ms`
- L=400: recall@10 `0.9970`, NDCG `0.9999`, mean `58.89 ms`
- L=800: recall@10 `0.9975`, NDCG `0.9999`, mean `68.90 ms`

### `latency-sidecar-early-stop-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11098-task29-diskann-early-stop-latency/artifacts/latency-sidecar-early-stop-cli.log bench latency --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --iterations 200 --concurrency 1 --force-index --sample-backend-memory --log-output review/11098-task29-diskann-early-stop-latency/artifacts/latency-sidecar-early-stop-table.log`

Timestamp: 2026-04-30 local.

Key result rows:

- L=64: mean `48.5 ms`, p50 `47.8 ms`, p95 `54.1 ms`, p99 `57.0 ms`, HWM `65024 KiB`
- L=128: mean `54.1 ms`, p50 `50.3 ms`, p95 `76.3 ms`, p99 `88.7 ms`, HWM `64544 KiB`
- L=200: mean `58.5 ms`, p50 `55.9 ms`, p95 `75.0 ms`, p99 `90.1 ms`, HWM `64544 KiB`
- L=400: mean `61.7 ms`, p50 `61.2 ms`, p95 `74.6 ms`, p99 `82.9 ms`, HWM `65268 KiB`
- L=800: mean `67.7 ms`, p50 `66.7 ms`, p95 `76.9 ms`, p99 `80.0 ms`, HWM `66640 KiB`
