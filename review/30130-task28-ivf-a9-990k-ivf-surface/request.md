# Task 28 IVF A9 990k IVF Selected Surface

## Scope

This packet records the first isolated 990k IVF selected-point surface for A9.

The surface is copied from the existing DBPedia 990k anchor into a one-index-per-table prefix so the IVF measurements cannot route through the existing anchor HNSW index.

Fixture:

- prefix: `task28_ivf_pqg990k_g8_n128`
- rows: 990000 corpus, 10000 queries
- quantizer: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- scan point: `nprobe=48`, `rerank_width=500`
- exact recall query cap: 100 queries

## Result

| metric | result |
|---|---:|
| corpus copy/load | 350927.311 ms |
| query copy/load | 1910.149 ms |
| IVF build time | 2033834.524 ms (33:53.835) |
| IVF index size | 185491456 bytes (177 MB) |
| recall@10 | 0.9860 |
| NDCG@10 | 0.9990 |
| recall@100 | 0.9509 |
| NDCG@100 | 0.9979 |
| recall run mean q-time, k=10 | 1066.42 ms |
| recall run mean q-time, k=100 | 1088.61 ms |
| latency p50/p95/p99 | 1029.2 / 1169.1 / 1224.4 ms |
| latency min/max | 869.0 / 1310.4 ms |
| latency RSS/HWM peak | 162636 / 162636 KB |

Cache state: warm local PG18 database. No OS or Postgres cache drop was run between build, recall, and latency measurements.

## Interpretation

This fills the largest A9 IVF-side scale gap: current-head IVF selected point on the 990k anchor shape with build time, index size, recall@10, recall@100, p50/p95/p99 latency, and memory high-water from the latency harness.

Recall remains reasonable at 990k for the selected PQ-FastScan g8 point. The bigger issue is latency: roughly 1.0-1.2s/query at `nprobe=48`, which means further IVF work should prioritize score-volume reduction, narrower candidate/rerank paths, and/or a lower-latency operating point rather than more broad HNSW comparison in this slice.

The exact-recall harness itself is also expensive at 990k because it rematerializes the full raw 1536D corpus for each recall run. That does not change the IVF query results, but it should be considered when planning the next measurement loop.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/anchor_schema_inventory.sql --raw --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/anchor_schema_inventory.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/build_pqg8_990k_n128_w500.sql --raw --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/build_pqg8_990k_n128_w500.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/recall10_pqg8_990k_n128_w500.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/latency_pqg8_990k_n128_w500.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg990k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --log-output review/30130-task28-ivf-a9-990k-ivf-surface/artifacts/recall100_pqg8_990k_n128_w500.log`

## Artifacts

- `artifacts/anchor_schema_inventory.sql`
- `artifacts/anchor_schema_inventory.log`
- `artifacts/build_pqg8_990k_n128_w500.sql`
- `artifacts/build_pqg8_990k_n128_w500.log`
- `artifacts/recall10_pqg8_990k_n128_w500.log`
- `artifacts/latency_pqg8_990k_n128_w500.log`
- `artifacts/recall100_pqg8_990k_n128_w500.log`
- `artifacts/build_progress_*.log`
- `artifacts/build_size_progress_001.log`
- `artifacts/manifest.md`
