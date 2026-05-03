# Task 31 M5 PQ-FastScan g8 100k n128 w500 Baseline Artifact Manifest

Head SHA: `adf4ca75c7a5295e75dc6d7413ef475af292f878`

Packet/topic: `review/30173-task31-m5-pqg8-100k-n128-w500-baseline`

Timestamp: `2026-05-03T05:07:38Z`

Machine: Task 31 M5 laptop from packet `30162`, Apple M5 Pro, macOS local PG18
pgrx environment.

Database target: `postgres`, socket directory `/Users/peter/.pgrx`, port `28818`.

CLI path: `/Users/peter/.cargo/bin/ecaz`

Surface:

- Corpus source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_corpus.tsv`
- Query source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_queries.tsv`
- Manifest source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_manifest.json`
- Corpus rows: `100000`
- Query rows: `1000`
- Dimensions: `1536`
- Corpus SHA256: `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`
- Query SHA256: `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Loaded prefix: `task31_m5_real100k_pqg8_n128`
- Profile: `ec_ivf`
- Storage format: `pq_fastscan`
- PQ group size: `8`
- `nlists`: `128`
- `nprobe`: `48`
- Rerank mode: `heap_f32`
- Rerank width: `500`
- Surface isolation: one-index-per-table Task 31 prefix; corpus table has one
  `ec_ivf` index plus its btree primary key.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Artifacts

### `load_real100k_pqg8_n128_w500_allow_manifest_mismatch.log`

- Lane: Task 31 M5 real 100k n128 w500 load/build.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/load_real100k_pqg8_n128_w500_allow_manifest_mismatch.log corpus load --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --corpus-file data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_corpus.tsv --queries-file data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_queries.tsv --manifest-file data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_manifest.json --allow-manifest-mismatch --reloption storage_format=pq_fastscan --reloption pq_group_size=8 --reloption nlists=128 --reloption nprobe=48 --reloption rerank=heap_f32 --reloption rerank_width=500`
- Key result lines:
  - Manifest verification warned because staged manifest prefix was
    `ec_hnsw_real_100k` while load prefix was
    `task31_m5_real100k_pqg8_n128`.
  - `copied corpus table task31_m5_real100k_pqg8_n128_corpus in 15.02s`
  - `encoded corpus table task31_m5_real100k_pqg8_n128_corpus in 5.89s`
  - `copied queries table task31_m5_real100k_pqg8_n128_queries in 183.48ms`
  - `built task31_m5_real100k_pqg8_n128_idx in 18.69s`
  - `completed prefix task31_m5_real100k_pqg8_n128 in 45.76s`

### `recall10_real100k_pqg8_n128_p48_w500.log`, `truth_real100k_n128_k10.json`

- Lane: Task 31 M5 real 100k n128 w500 recall@10.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k10.json --log-output review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/recall10_real100k_pqg8_n128_p48_w500.log`
- Key result: `recall@10=0.9820`, `ndcg@10=0.9981`, `mean q-time=6.72 ms`.

### `recall100_real100k_pqg8_n128_p48_w500.log`, `truth_real100k_n128_k100.json`

- Lane: Task 31 M5 real 100k n128 w500 recall@100.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48 --rerank-width 500 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k100.json --log-output review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/recall100_real100k_pqg8_n128_p48_w500.log`
- Key result: `recall@100=0.9436`, `ndcg@100=0.9970`, `mean q-time=7.19 ms`.

### `latency_real100k_pqg8_n128_p48_w500.log`

- Lane: Task 31 M5 real 100k n128 w500 latency.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/latency_real100k_pqg8_n128_p48_w500.log`
- Key result: `mean=6.79 ms`, `p50=6.80 ms`, `p95=7.37 ms`, `p99=7.63 ms`, `memory_samples=0`.

### `storage_real100k_pqg8_n128.log`

- Lane: Task 31 M5 real 100k n128 w500 storage.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/storage_real100k_pqg8_n128.log bench storage --prefix task31_m5_real100k_pqg8_n128`
- Key result: `rows=100000`, table total `1.6 GiB`, indexes `23.7 MiB`, IVF index `19.4 MiB`, IVF index per row `202.9 B`.

### `explain_real100k_pqg8_n128_p48_w500.sql`

- Lane: Task 31 M5 real 100k n128 w500 EXPLAIN/counter SQL.
- Purpose: packet-local SQL for representative counter capture.

### `explain_real100k_pqg8_n128_p48_w500.log`

- Lane: Task 31 M5 real 100k n128 w500 EXPLAIN/counter capture.
- Command:
  `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --port 28818 --raw --file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/explain_real100k_pqg8_n128_p48_w500.sql --log-output review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/explain_real100k_pqg8_n128_p48_w500.log`
- Key result: `index_bytes=20291584`, `Execution Time=11.250 ms`, `Centroid Scores=128`, `Selected Lists=48`, `Posting Pages Read=817`, `Postings Visited=34896`, `Postings Scored=2748`, `Postings Pruned By Bound=32148`, `Candidates Inserted=2748`, `Rerank Rows=500`, `Filtered Duplicates=0`.
