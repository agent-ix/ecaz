# Task 31 M5 100k Candidate Repeatability Artifact Manifest

Head SHA: `07e8dc653914272da9c9e6237ee472dd10de770e`

Packet/topic: `review/30176-task31-m5-100k-candidate-repeatability`

Timestamp: `2026-05-03T05:21:48Z`

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
- `nprobe`: `96`
- Rerank mode: `heap_f32`
- Rerank widths: `500`, `1000`
- Surface isolation: one-index-per-table Task 31 prefix from packet `30173`.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Artifacts

### `latency_repeat_real100k_pqg8_n128_p96_w500.log`

- Lane: Task 31 M5 real 100k candidate repeat latency, balanced point.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 96 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30176-task31-m5-100k-candidate-repeatability/artifacts/latency_repeat_real100k_pqg8_n128_p96_w500.log`
- Key result: `p50=10.8 ms`, `p95=11.4 ms`, `p99=11.9 ms`, `memory_samples=0`.

### `latency_repeat_real100k_pqg8_n128_p96_w1000.log`

- Lane: Task 31 M5 real 100k candidate repeat latency, quality-biased point.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 96 --rerank-width 1000 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30176-task31-m5-100k-candidate-repeatability/artifacts/latency_repeat_real100k_pqg8_n128_p96_w1000.log`
- Key result: `p50=12.9 ms`, `p95=13.9 ms`, `p99=14.3 ms`, `memory_samples=0`.

### `recall10_repeat_real100k_pqg8_n128_p96_w500.log`

- Lane: Task 31 M5 real 100k candidate repeat recall@10, balanced point.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 96 --rerank-width 500 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k10.json --log-output review/30176-task31-m5-100k-candidate-repeatability/artifacts/recall10_repeat_real100k_pqg8_n128_p96_w500.log`
- Key result: `recall@10=0.9980`, `ndcg@10=0.9997`.

### `recall100_repeat_real100k_pqg8_n128_p96_w500.log`

- Lane: Task 31 M5 real 100k candidate repeat recall@100, balanced point.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 96 --rerank-width 500 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k100.json --log-output review/30176-task31-m5-100k-candidate-repeatability/artifacts/recall100_repeat_real100k_pqg8_n128_p96_w500.log`
- Key result: `recall@100=0.9676`, `ndcg@100=0.9991`.

### `recall10_repeat_real100k_pqg8_n128_p96_w1000.log`

- Lane: Task 31 M5 real 100k candidate repeat recall@10, quality-biased point.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 96 --rerank-width 1000 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k10.json --log-output review/30176-task31-m5-100k-candidate-repeatability/artifacts/recall10_repeat_real100k_pqg8_n128_p96_w1000.log`
- Key result: `recall@10=0.9980`, `ndcg@10=0.9997`.

### `recall100_repeat_real100k_pqg8_n128_p96_w1000.log`

- Lane: Task 31 M5 real 100k candidate repeat recall@100, quality-biased point.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 96 --rerank-width 1000 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k100.json --log-output review/30176-task31-m5-100k-candidate-repeatability/artifacts/recall100_repeat_real100k_pqg8_n128_p96_w1000.log`
- Key result: `recall@100=0.9920`, `ndcg@100=0.9997`.
