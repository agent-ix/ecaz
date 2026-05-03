# Task 31 M5 100k n128 Rerank Sweep Artifact Manifest

Head SHA: `743991253b0cb572dafedb69b3c64e2167beb2db`

Packet/topic: `review/30175-task31-m5-100k-n128-rerank-sweep`

Timestamp: `2026-05-03T05:12:18Z`

Machine: Task 31 M5 laptop from packet `30162`, Apple M5 Pro, macOS local PG18
pgrx environment.

Database target: `postgres`, socket directory `/Users/peter/.pgrx`, port `28818`.

CLI path: `/Users/peter/.cargo/bin/ecaz`

Surface:

- Loaded prefix: `task31_m5_real100k_pqg8_n128`
- Corpus source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_corpus.tsv`
- Query source: `data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_queries.tsv`
- Corpus rows: `100000`
- Query rows: `1000`
- Dimensions: `1536`
- Corpus SHA256: `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`
- Query SHA256: `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Profile: `ec_ivf`
- Storage format: `pq_fastscan`
- PQ group size: `8`
- `nlists`: `128`
- Swept `nprobe`: `80,96`
- Swept rerank width: `750,1000`
- Surface isolation: one-index-per-table Task 31 prefix from packet `30173`.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Artifacts

### `recall10_real100k_pqg8_n128_p80_96_w750.log`

- Lane: Task 31 M5 real 100k n128 recall@10 at width 750.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 80,96 --rerank-width 750 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k10.json --log-output review/30175-task31-m5-100k-n128-rerank-sweep/artifacts/recall10_real100k_pqg8_n128_p80_96_w750.log`
- Key result: `p80 recall@10=0.9960`, `p96 recall@10=0.9980`.

### `recall100_real100k_pqg8_n128_p80_96_w750.log`

- Lane: Task 31 M5 real 100k n128 recall@100 at width 750.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 80,96 --rerank-width 750 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k100.json --log-output review/30175-task31-m5-100k-n128-rerank-sweep/artifacts/recall100_real100k_pqg8_n128_p80_96_w750.log`
- Key result: `p80 recall@100=0.9805`, `p96 recall@100=0.9843`.

### `recall10_real100k_pqg8_n128_p80_96_w1000.log`

- Lane: Task 31 M5 real 100k n128 recall@10 at width 1000.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 80,96 --rerank-width 1000 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k10.json --log-output review/30175-task31-m5-100k-n128-rerank-sweep/artifacts/recall10_real100k_pqg8_n128_p80_96_w1000.log`
- Key result: `p80 recall@10=0.9960`, `p96 recall@10=0.9980`.

### `recall100_real100k_pqg8_n128_p80_96_w1000.log`

- Lane: Task 31 M5 real 100k n128 recall@100 at width 1000.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 80,96 --rerank-width 1000 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k100.json --log-output review/30175-task31-m5-100k-n128-rerank-sweep/artifacts/recall100_real100k_pqg8_n128_p80_96_w1000.log`
- Key result: `p80 recall@100=0.9880`, `p96 recall@100=0.9920`.

### `latency_real100k_pqg8_n128_p80_96_w750.log`

- Lane: Task 31 M5 real 100k n128 latency at width 750.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 80,96 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30175-task31-m5-100k-n128-rerank-sweep/artifacts/latency_real100k_pqg8_n128_p80_96_w750.log`
- Key result: `p80 p50=10.6 ms p95=11.5 ms`, `p96 p50=11.9 ms p95=12.7 ms`.

### `latency_real100k_pqg8_n128_p80_96_w1000.log`

- Lane: Task 31 M5 real 100k n128 latency at width 1000.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 80,96 --rerank-width 1000 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30175-task31-m5-100k-n128-rerank-sweep/artifacts/latency_real100k_pqg8_n128_p80_96_w1000.log`
- Key result: `p80 p50=11.6 ms p95=12.9 ms`, `p96 p50=13.1 ms p95=13.8 ms`.

### `explain_real100k_pqg8_n128_p96_w1000.sql`

- Lane: Task 31 M5 real 100k n128 p96 w1000 EXPLAIN/counter SQL.
- Purpose: packet-local SQL for representative counter capture.

### `explain_real100k_pqg8_n128_p96_w1000.log`

- Lane: Task 31 M5 real 100k n128 p96 w1000 EXPLAIN/counter capture.
- Command:
  `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --port 28818 --raw --file review/30175-task31-m5-100k-n128-rerank-sweep/artifacts/explain_real100k_pqg8_n128_p96_w1000.sql --log-output review/30175-task31-m5-100k-n128-rerank-sweep/artifacts/explain_real100k_pqg8_n128_p96_w1000.log`
- Key result: `Execution Time=20.366 ms`, `Selected Lists=96`, `Posting Pages Read=1815`, `Postings Visited=77760`, `Postings Scored=6509`, `Postings Pruned By Bound=71251`, `Candidates Inserted=6509`, `Rerank Rows=1000`.
