# Task 31 M5 PQ-FastScan g8 100k Load Baseline Artifact Manifest

Head SHA: `fbb4d86d704f2667d5f57f1407b5d8c8993daee8`

Packet/topic: `review/30172-task31-m5-pqg8-100k-load-baseline`

Timestamp: `2026-05-03T04:57:39Z`

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
- Loaded prefix: `task31_m5_real100k_pqg8_n64`
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

### `load_real100k_pqg8_n64_w750_allow_manifest_mismatch.log`

- Lane: Task 31 M5 real 100k load/build.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30172-task31-m5-pqg8-100k-load-baseline/artifacts/load_real100k_pqg8_n64_w750_allow_manifest_mismatch.log corpus load --prefix task31_m5_real100k_pqg8_n64 --profile ec_ivf --corpus-file data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_corpus.tsv --queries-file data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_queries.tsv --manifest-file data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_manifest.json --allow-manifest-mismatch --reloption storage_format=pq_fastscan --reloption pq_group_size=8 --reloption nlists=64 --reloption nprobe=48 --reloption rerank=heap_f32 --reloption rerank_width=750`
- Key result lines:
  - Manifest verification warned because staged manifest prefix was
    `ec_hnsw_real_100k` while load prefix was
    `task31_m5_real100k_pqg8_n64`.
  - `corpus: 100000 rows, sha256=07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`
  - `queries: 1000 rows, sha256=a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
  - `copied corpus table task31_m5_real100k_pqg8_n64_corpus in 19.17s`
  - `encoded corpus table task31_m5_real100k_pqg8_n64_corpus in 7.89s`
  - `copied queries table task31_m5_real100k_pqg8_n64_queries in 174.47ms`
  - `built task31_m5_real100k_pqg8_n64_idx in 9.55s`
  - `completed prefix task31_m5_real100k_pqg8_n64 in 42.92s`

### `recall10_real100k_pqg8_n64_p48_w750.log`, `truth_real100k_k10.json`

- Lane: Task 31 M5 real 100k recall.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n64 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 750 --force-index --truth-cache-file review/30172-task31-m5-pqg8-100k-load-baseline/artifacts/truth_real100k_k10.json --log-output review/30172-task31-m5-pqg8-100k-load-baseline/artifacts/recall10_real100k_pqg8_n64_p48_w750.log`
- Key result:
  - `nprobe=48`
  - `recall@10=0.9940`
  - `ndcg@10=0.9996`
  - `mean q-time=11.72 ms`

### `latency_real100k_pqg8_n64_p48_w750.log`

- Lane: Task 31 M5 real 100k latency.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real100k_pqg8_n64 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30172-task31-m5-pqg8-100k-load-baseline/artifacts/latency_real100k_pqg8_n64_p48_w750.log`
- Key result:
  - `count=100`
  - `mean=11.7 ms`
  - `p50=11.7 ms`
  - `p95=12.7 ms`
  - `p99=13.3 ms`
  - `memory_samples=0`
- Interpretation: backend memory sampling did not capture a sample on this
  surface. No HWM claim is made.

### `storage_real100k_pqg8_n64.log`

- Lane: Task 31 M5 real 100k storage.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30172-task31-m5-pqg8-100k-load-baseline/artifacts/storage_real100k_pqg8_n64.log bench storage --prefix task31_m5_real100k_pqg8_n64`
- Key result:
  - `rows=100000`
  - `table (heap + toast + fsm/vm)=1.6 GiB`
  - `indexes=22.9 MiB`
  - `task31_m5_real100k_pqg8_n64_idx=18.6 MiB`
  - IVF index per row `195.4 B`

### `explain_real100k_pqg8_n64_p48_w750.sql`

- Lane: Task 31 M5 real 100k EXPLAIN/counter SQL.
- Purpose: packet-local SQL for representative counter capture.

### `explain_real100k_pqg8_n64_p48_w750.log`

- Lane: Task 31 M5 real 100k EXPLAIN/counter capture.
- Command:
  `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --port 28818 --raw --file review/30172-task31-m5-pqg8-100k-load-baseline/artifacts/explain_real100k_pqg8_n64_p48_w750.sql --log-output review/30172-task31-m5-pqg8-100k-load-baseline/artifacts/explain_real100k_pqg8_n64_p48_w750.log`
- Key result:
  - `index_bytes=19537920`
  - `Execution Time=19.267 ms`
  - `Shared Hit Blocks=3656`
  - `Shared Read Blocks=3122`
  - `Centroid Scores=64`
  - `Selected Lists=48`
  - `Posting Pages Read=1753`
  - `Postings Visited=76022`
  - `Postings Scored=5072`
  - `Postings Pruned By Bound=70950`
  - `Candidates Inserted=5072`
  - `Rerank Rows=750`
  - `Filtered Duplicates=0`
