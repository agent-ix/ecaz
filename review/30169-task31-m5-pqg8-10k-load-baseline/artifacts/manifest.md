# Task 31 M5 PQ-FastScan g8 10k Load Baseline Artifact Manifest

Head SHA: `47214b1a72d87c7c9ca211f6505ffecdd1505e9f`

Packet/topic: `review/30169-task31-m5-pqg8-10k-load-baseline`

Timestamp: `2026-05-03T03:49:36Z`

Machine: Task 31 M5 laptop from packet `30162`, Apple M5 Pro, macOS local PG18
pgrx environment.

Database target: `postgres`, socket directory `/Users/peter/.pgrx`, port `28818`.

CLI path: `/Users/peter/.cargo/bin/ecaz`

Surface:

- Corpus source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_corpus.tsv`
- Query source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_queries.tsv`
- Manifest source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_manifest.json`
- Corpus rows: `10000`
- Query rows: `200`
- Dimensions: `1536`
- Corpus SHA256: `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`
- Query SHA256: `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- Loaded prefix: `task31_m5_real10k_pqg8_n64`
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

### `load_real10k_pqg8_n64_w750.log`

- Lane: Task 31 M5 real 10k load precheck.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/load_real10k_pqg8_n64_w750.log corpus load --prefix task31_m5_real10k_pqg8_n64 --profile ec_ivf --corpus-file data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_corpus.tsv --queries-file data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_queries.tsv --manifest-file data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_manifest.json --reloption storage_format=pq_fastscan --reloption pq_group_size=8 --reloption nlists=64 --reloption nprobe=48 --reloption rerank=heap_f32 --reloption rerank_width=750`
- Key result:
  - Manifest verification failed because staged manifest prefix was
    `ec_hnsw_real_10k` while load prefix was `task31_m5_real10k_pqg8_n64`.
  - Corpus/query hashes matched the staged manifest.
  - No table/index load claim is made from this failed precheck.

### `load_real10k_pqg8_n64_w750_allow_manifest_mismatch.log`

- Lane: Task 31 M5 real 10k load/build.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/load_real10k_pqg8_n64_w750_allow_manifest_mismatch.log corpus load --prefix task31_m5_real10k_pqg8_n64 --profile ec_ivf --corpus-file data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_corpus.tsv --queries-file data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_queries.tsv --manifest-file data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_manifest.json --allow-manifest-mismatch --reloption storage_format=pq_fastscan --reloption pq_group_size=8 --reloption nlists=64 --reloption nprobe=48 --reloption rerank=heap_f32 --reloption rerank_width=750`
- Key result lines:
  - `copied corpus table task31_m5_real10k_pqg8_n64_corpus in 1.95s`
  - `encoded corpus table task31_m5_real10k_pqg8_n64_corpus in 646.17ms`
  - `copied queries table task31_m5_real10k_pqg8_n64_queries in 32.65ms`
  - `built task31_m5_real10k_pqg8_n64_idx in 4.29s`
  - `completed prefix task31_m5_real10k_pqg8_n64 in 7.55s`

### `recall10_real10k_pqg8_n64_p48_w750.log`, `truth_real10k_k10.json`

- Lane: Task 31 M5 real 10k recall.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real10k_pqg8_n64 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 750 --force-index --truth-cache-file review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/truth_real10k_k10.json --log-output review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/recall10_real10k_pqg8_n64_p48_w750.log`
- Key result:
  - `nprobe=48`
  - `recall@10=1.0000`
  - `ndcg@10=1.0000`
  - `mean q-time=3.05 ms`

### `latency_real10k_pqg8_n64_p48_w750.log`

- Lane: Task 31 M5 real 10k latency.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real10k_pqg8_n64 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/latency_real10k_pqg8_n64_p48_w750.log`
- Key result:
  - `count=100`
  - `mean=3.06 ms`
  - `p50=2.96 ms`
  - `p95=3.51 ms`
  - `p99=4.23 ms`
  - `memory_samples=0`

### `latency_real10k_pqg8_n64_p48_w750_mem1ms.log`

- Lane: Task 31 M5 real 10k latency memory-sampling retry.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real10k_pqg8_n64 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 1 --log-output review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/latency_real10k_pqg8_n64_p48_w750_mem1ms.log`
- Key result:
  - `count=100`
  - `mean=3.14 ms`
  - `p50=3.00 ms`
  - `p95=3.88 ms`
  - `p99=4.51 ms`
  - `memory_samples=0`
- Interpretation: backend memory sampling did not capture a sample on this very
  fast 10k surface, even at 1ms. No HWM claim is made.

### `storage_real10k_pqg8_n64.log`

- Lane: Task 31 M5 real 10k storage.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/storage_real10k_pqg8_n64.log bench storage --prefix task31_m5_real10k_pqg8_n64`
- Key result:
  - `rows=10000`
  - `table (heap + toast + fsm/vm)=159.4 MiB`
  - `indexes=3.1 MiB`
  - `task31_m5_real10k_pqg8_n64_idx=2.6 MiB`
  - IVF index per row `277.7 B`

### `explain_real10k_pqg8_n64_p48_w750.sql`

- Lane: Task 31 M5 real 10k EXPLAIN/counter SQL.
- Purpose: packet-local SQL for representative counter capture.

### `explain_real10k_pqg8_n64_p48_w750.log`

- Lane: Task 31 M5 real 10k EXPLAIN/counter capture.
- Command:
  `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --port 28818 --raw --file review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/explain_real10k_pqg8_n64_p48_w750.sql --log-output review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/explain_real10k_pqg8_n64_p48_w750.log`
- Key result:
  - `index_bytes=2777088`
  - `Execution Time=6.337 ms`
  - `Shared Hit Blocks=4461`
  - `Shared Read Blocks=0`
  - `Centroid Scores=64`
  - `Selected Lists=48`
  - `Posting Pages Read=198`
  - `Postings Visited=7638`
  - `Postings Scored=2039`
  - `Postings Pruned By Bound=5599`
  - `Candidates Inserted=2039`
  - `Rerank Rows=750`
  - `Filtered Duplicates=0`
