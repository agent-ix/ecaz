# Task 31 M5 PQ-FastScan g8 25k Load Baseline Artifact Manifest

Head SHA: `dca07cc9af4063a2fdbc6c5ce3237015db652178`

Packet/topic: `review/30170-task31-m5-pqg8-25k-load-baseline`

Timestamp: `2026-05-03T04:22:06Z`

Machine: Task 31 M5 laptop from packet `30162`, Apple M5 Pro, macOS local PG18
pgrx environment.

Database target: `postgres`, socket directory `/Users/peter/.pgrx`, port `28818`.

CLI path: `/Users/peter/.cargo/bin/ecaz`

Surface:

- Corpus source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_25k_corpus.tsv`
- Query source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_25k_queries.tsv`
- Manifest source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_25k_manifest.json`
- Corpus rows: `25000`
- Query rows: `500`
- Dimensions: `1536`
- Corpus SHA256: `234739ca91125db9d68052fe65380c32b9f41b42aa339320d77915680197a405`
- Query SHA256: `80548c67c965dc8f22e793d0ec7af78c96d23e60793fd5c41311a5543b64d2f8`
- Loaded prefix: `task31_m5_real25k_pqg8_n64`
- Profile: `ec_ivf`
- Storage format: `pq_fastscan`
- PQ group size: `8`
- `nlists`: `64`
- `nprobe`: `48`
- Rerank mode: `heap_f32`
- Rerank width: `750`
- Surface isolation: one-index-per-table Task 31 prefix; corpus table has one
  `ec_ivf` index plus its btree primary key.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Artifacts

### `load_real25k_pqg8_n64_w750_allow_manifest_mismatch.log`

- Lane: Task 31 M5 real 25k load/build.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30170-task31-m5-pqg8-25k-load-baseline/artifacts/load_real25k_pqg8_n64_w750_allow_manifest_mismatch.log corpus load --prefix task31_m5_real25k_pqg8_n64 --profile ec_ivf --corpus-file data/task31_m5_dbpedia_staged/ec_hnsw_real_25k_corpus.tsv --queries-file data/task31_m5_dbpedia_staged/ec_hnsw_real_25k_queries.tsv --manifest-file data/task31_m5_dbpedia_staged/ec_hnsw_real_25k_manifest.json --allow-manifest-mismatch --reloption storage_format=pq_fastscan --reloption pq_group_size=8 --reloption nlists=64 --reloption nprobe=48 --reloption rerank=heap_f32 --reloption rerank_width=750`
- Key result lines:
  - Manifest verification warned because staged manifest prefix was
    `ec_hnsw_real_25k` while load prefix was
    `task31_m5_real25k_pqg8_n64`.
  - `corpus: 25000 rows, sha256=234739ca91125db9d68052fe65380c32b9f41b42aa339320d77915680197a405`
  - `queries: 500 rows, sha256=80548c67c965dc8f22e793d0ec7af78c96d23e60793fd5c41311a5543b64d2f8`
  - `copied corpus table task31_m5_real25k_pqg8_n64_corpus in 3.79s`
  - `encoded corpus table task31_m5_real25k_pqg8_n64_corpus in 765.04ms`
  - `copied queries table task31_m5_real25k_pqg8_n64_queries in 74.30ms`
  - `built task31_m5_real25k_pqg8_n64_idx in 5.16s`
  - `completed prefix task31_m5_real25k_pqg8_n64 in 11.33s`

### `recall10_real25k_pqg8_n64_p48_w750.log`, `truth_real25k_k10.json`

- Lane: Task 31 M5 real 25k recall.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real25k_pqg8_n64 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 750 --force-index --truth-cache-file review/30170-task31-m5-pqg8-25k-load-baseline/artifacts/truth_real25k_k10.json --log-output review/30170-task31-m5-pqg8-25k-load-baseline/artifacts/recall10_real25k_pqg8_n64_p48_w750.log`
- Key result:
  - `nprobe=48`
  - `recall@10=0.9990`
  - `ndcg@10=1.0000`
  - `mean q-time=4.65 ms`

### `latency_real25k_pqg8_n64_p48_w750.log`

- Lane: Task 31 M5 real 25k latency.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real25k_pqg8_n64 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30170-task31-m5-pqg8-25k-load-baseline/artifacts/latency_real25k_pqg8_n64_p48_w750.log`
- Key result:
  - `count=100`
  - `mean=4.87 ms`
  - `p50=4.78 ms`
  - `p95=5.53 ms`
  - `p99=6.12 ms`
  - `memory_samples=0`
- Interpretation: backend memory sampling did not capture a sample on this
  still-fast 25k surface. No HWM claim is made.

### `storage_real25k_pqg8_n64.log`

- Lane: Task 31 M5 real 25k storage.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30170-task31-m5-pqg8-25k-load-baseline/artifacts/storage_real25k_pqg8_n64.log bench storage --prefix task31_m5_real25k_pqg8_n64`
- Key result:
  - `rows=25000`
  - `table (heap + toast + fsm/vm)=398.4 MiB`
  - `indexes=6.4 MiB`
  - `task31_m5_real25k_pqg8_n64_idx=5.3 MiB`
  - IVF index per row `223.2 B`

### `explain_real25k_pqg8_n64_p48_w750.sql`

- Lane: Task 31 M5 real 25k EXPLAIN/counter SQL.
- Purpose: packet-local SQL for representative counter capture.

### `explain_real25k_pqg8_n64_p48_w750.log`

- Lane: Task 31 M5 real 25k EXPLAIN/counter capture.
- Command:
  `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --port 28818 --raw --file review/30170-task31-m5-pqg8-25k-load-baseline/artifacts/explain_real25k_pqg8_n64_p48_w750.sql --log-output review/30170-task31-m5-pqg8-25k-load-baseline/artifacts/explain_real25k_pqg8_n64_p48_w750.log`
- Key result:
  - `index_bytes=5578752`
  - `Execution Time=8.648 ms`
  - `Shared Hit Blocks=5141`
  - `Shared Read Blocks=309`
  - `Centroid Scores=64`
  - `Selected Lists=48`
  - `Posting Pages Read=424`
  - `Postings Visited=17547`
  - `Postings Scored=2705`
  - `Postings Pruned By Bound=14842`
  - `Candidates Inserted=2705`
  - `Rerank Rows=750`
  - `Filtered Duplicates=0`
