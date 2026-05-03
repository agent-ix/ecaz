# Task 31 M5 PQ-FastScan g8 50k Load Baseline Artifact Manifest

Head SHA: `0644effb6b0fa066b1673c48b6b461488373363e`

Packet/topic: `review/30171-task31-m5-pqg8-50k-load-baseline`

Timestamp: `2026-05-03T04:27:41Z`

Machine: Task 31 M5 laptop from packet `30162`, Apple M5 Pro, macOS local PG18
pgrx environment.

Database target: `postgres`, socket directory `/Users/peter/.pgrx`, port `28818`.

CLI path: `/Users/peter/.cargo/bin/ecaz`

Surface:

- Corpus source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_50k_corpus.tsv`
- Query source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_50k_queries.tsv`
- Manifest source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_50k_manifest.json`
- Corpus rows: `50000`
- Query rows: `1000`
- Dimensions: `1536`
- Corpus SHA256: `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133`
- Query SHA256: `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`
- Loaded prefix: `task31_m5_real50k_pqg8_n64`
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

### `load_real50k_pqg8_n64_w750_allow_manifest_mismatch.log`

- Lane: Task 31 M5 real 50k load/build.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30171-task31-m5-pqg8-50k-load-baseline/artifacts/load_real50k_pqg8_n64_w750_allow_manifest_mismatch.log corpus load --prefix task31_m5_real50k_pqg8_n64 --profile ec_ivf --corpus-file data/task31_m5_dbpedia_staged/ec_hnsw_real_50k_corpus.tsv --queries-file data/task31_m5_dbpedia_staged/ec_hnsw_real_50k_queries.tsv --manifest-file data/task31_m5_dbpedia_staged/ec_hnsw_real_50k_manifest.json --allow-manifest-mismatch --reloption storage_format=pq_fastscan --reloption pq_group_size=8 --reloption nlists=64 --reloption nprobe=48 --reloption rerank=heap_f32 --reloption rerank_width=750`
- Key result lines:
  - Manifest verification warned because staged manifest prefix was
    `ec_hnsw_real_50k` while load prefix was
    `task31_m5_real50k_pqg8_n64`.
  - `corpus: 50000 rows, sha256=56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133`
  - `queries: 1000 rows, sha256=95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`
  - `copied corpus table task31_m5_real50k_pqg8_n64_corpus in 7.54s`
  - `encoded corpus table task31_m5_real50k_pqg8_n64_corpus in 1.58s`
  - `copied queries table task31_m5_real50k_pqg8_n64_queries in 148.28ms`
  - `built task31_m5_real50k_pqg8_n64_idx in 6.59s`
  - `completed prefix task31_m5_real50k_pqg8_n64 in 18.86s`

### `recall10_real50k_pqg8_n64_p48_w750.log`, `truth_real50k_k10.json`

- Lane: Task 31 M5 real 50k recall.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real50k_pqg8_n64 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 --rerank-width 750 --force-index --truth-cache-file review/30171-task31-m5-pqg8-50k-load-baseline/artifacts/truth_real50k_k10.json --log-output review/30171-task31-m5-pqg8-50k-load-baseline/artifacts/recall10_real50k_pqg8_n64_p48_w750.log`
- Key result:
  - `nprobe=48`
  - `recall@10=1.0000`
  - `ndcg@10=1.0000`
  - `mean q-time=6.82 ms`

### `latency_real50k_pqg8_n64_p48_w750.log`

- Lane: Task 31 M5 real 50k latency.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real50k_pqg8_n64 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30171-task31-m5-pqg8-50k-load-baseline/artifacts/latency_real50k_pqg8_n64_p48_w750.log`
- Key result:
  - `count=100`
  - `mean=6.69 ms`
  - `p50=6.63 ms`
  - `p95=7.24 ms`
  - `p99=7.73 ms`
  - `memory_samples=0`
- Interpretation: backend memory sampling did not capture a sample on this fast
  50k surface. No HWM claim is made.

### `storage_real50k_pqg8_n64.log`

- Lane: Task 31 M5 real 50k storage.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30171-task31-m5-pqg8-50k-load-baseline/artifacts/storage_real50k_pqg8_n64.log bench storage --prefix task31_m5_real50k_pqg8_n64`
- Key result:
  - `rows=50000`
  - `table (heap + toast + fsm/vm)=796.6 MiB`
  - `indexes=11.9 MiB`
  - `task31_m5_real50k_pqg8_n64_idx=9.7 MiB`
  - IVF index per row `203.8 B`

### `explain_real50k_pqg8_n64_p48_w750.sql`

- Lane: Task 31 M5 real 50k EXPLAIN/counter SQL.
- Purpose: packet-local SQL for representative counter capture.

### `explain_real50k_pqg8_n64_p48_w750.log`

- Lane: Task 31 M5 real 50k EXPLAIN/counter capture.
- Command:
  `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --port 28818 --raw --file review/30171-task31-m5-pqg8-50k-load-baseline/artifacts/explain_real50k_pqg8_n64_p48_w750.sql --log-output review/30171-task31-m5-pqg8-50k-load-baseline/artifacts/explain_real50k_pqg8_n64_p48_w750.log`
- Key result:
  - `index_bytes=10190848`
  - `Execution Time=12.542 ms`
  - `Shared Hit Blocks=3926`
  - `Shared Read Blocks=1909`
  - `Centroid Scores=64`
  - `Selected Lists=48`
  - `Posting Pages Read=810`
  - `Postings Visited=34639`
  - `Postings Scored=3895`
  - `Postings Pruned By Bound=30744`
  - `Candidates Inserted=3895`
  - `Rerank Rows=750`
  - `Filtered Duplicates=0`
