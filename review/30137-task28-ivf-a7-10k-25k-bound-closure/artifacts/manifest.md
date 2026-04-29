# Artifact Manifest

Measurement head SHA: `8b5d3c79e49ef15359cb4c8f62970f9b20e9c594`

Packet commit base SHA: `d0c91cc7c98f08eeafc0ea44872dcdd12df6361d`

Packet: `review/30137-task28-ivf-a7-10k-25k-bound-closure`

Environment:

- timestamp: `2026-04-28T20:27:31-07:00`
- OS: `Linux DESKTOP-BMB4AFO 6.6.87.2-microsoft-standard-WSL2 x86_64`
- CPU: `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`, 20 logical CPUs
- memory: 62 GiB total
- PostgreSQL: 18.3
- cache state: warm local development run; no explicit OS or PostgreSQL buffer-cache drop
- surface isolation: isolated one-index-per-table corpus prefixes

## `recall10_pqg8_10k_n64_w750_p48.log`

- lane: IVF A7 PQ-FastScan score-bound pruning, 10k frontier
- fixture: `task28_ivf_pqg10k_g8`, `storage_format=pq_fastscan`, `pq_group_size=8`, `nlists=64`, `nprobe=48`, `rerank=heap_f32`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 750 --force-index --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/recall10_pqg8_10k_n64_w750_p48.log`
- key result: `recall@10=0.9910`, `NDCG@10=0.9997`, `mean q-time=80.90 ms`

## `latency_pqg8_10k_n64_w750_p48.log`

- lane: IVF A7 PQ-FastScan score-bound pruning, 10k frontier
- fixture: `task28_ivf_pqg10k_g8`, `storage_format=pq_fastscan`, `pq_group_size=8`, `nlists=64`, `nprobe=48`, `rerank=heap_f32`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/latency_pqg8_10k_n64_w750_p48.log`
- key result: `count=100`, `mean=77.5 ms`, `p50=77.3 ms`, `p95=80.4 ms`, `p99=82.2 ms`, `rss_peak_kb=137244`, `hwm_peak_kb=137244`

## `recall10_pqg8_25k_n64_w750_p48.log`

- lane: IVF A7 PQ-FastScan score-bound pruning, 25k frontier
- fixture: `task28_ivf_pqg25k_g8`, `storage_format=pq_fastscan`, `pq_group_size=8`, `nlists=64`, `nprobe=48`, `rerank=heap_f32`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 750 --force-index --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/recall10_pqg8_25k_n64_w750_p48.log`
- key result: `recall@10=0.9940`, `NDCG@10=1.0000`, `mean q-time=116.48 ms`

## `latency_pqg8_25k_n64_w750_p48.log`

- lane: IVF A7 PQ-FastScan score-bound pruning, 25k frontier
- fixture: `task28_ivf_pqg25k_g8`, `storage_format=pq_fastscan`, `pq_group_size=8`, `nlists=64`, `nprobe=48`, `rerank=heap_f32`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg25k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/latency_pqg8_25k_n64_w750_p48.log`
- key result: `count=100`, `mean=117.4 ms`, `p50=116.8 ms`, `p95=123.7 ms`, `p99=125.7 ms`, `rss_peak_kb=156112`, `hwm_peak_kb=156112`

## `explain_10k_25k_n64_w750_p48.sql`

- lane: IVF A7 PQ-FastScan score-volume counters
- fixture: `task28_ivf_pqg10k_g8` and `task28_ivf_pqg25k_g8`, `nprobe=48`, `rerank_width=750`
- purpose: packet-local SQL for EXPLAIN counter capture

## `explain_10k_25k_n64_w750_p48.log`

- lane: IVF A7 PQ-FastScan score-volume counters
- fixture: `task28_ivf_pqg10k_g8` and `task28_ivf_pqg25k_g8`, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/explain_10k_25k_n64_w750_p48.sql --raw --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/explain_10k_25k_n64_w750_p48.log`
- key result, 10k: `index_bytes=2506752`, `Postings Visited=7578`, `Postings Scored=2293`, `Postings Pruned By Bound=5285`, `Rerank Rows=750`, `Execution Time=124.440 ms`
- key result, 25k: `index_bytes=5300224`, `Postings Visited=19750`, `Postings Scored=3494`, `Postings Pruned By Bound=16256`, `Rerank Rows=750`, `Execution Time=118.910 ms`

## `recall10_pqg8_10k_n128_w750_p48.log`

- lane: n128 sanity check, not the A7 frontier closure row
- fixture: `task28_ivf_pqg10k_g8_n128`, `storage_format=pq_fastscan`, `pq_group_size=8`, `nlists=128`, `nprobe=48`, `rerank=heap_f32`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 750 --force-index --log-output review/30137-task28-ivf-a7-10k-25k-bound-closure/artifacts/recall10_pqg8_10k_n128_w750_p48.log`
- key result: `recall@10=0.9600`, `NDCG@10=0.9969`, `mean q-time=73.38 ms`
