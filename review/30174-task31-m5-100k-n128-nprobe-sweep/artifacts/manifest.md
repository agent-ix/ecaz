# Task 31 M5 100k n128 nprobe Sweep Artifact Manifest

Head SHA: `adf4ca75c7a5295e75dc6d7413ef475af292f878`

Packet/topic: `review/30174-task31-m5-100k-n128-nprobe-sweep`

Timestamp: `2026-05-03T05:07:38Z`

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
- Rerank mode: `heap_f32`
- Rerank width: `500`
- Swept `nprobe`: `40,48,56,64,80,96`
- Surface isolation: one-index-per-table Task 31 prefix from packet `30173`.
- Cache state: warm local development run; no explicit OS or PostgreSQL buffer
  cache drop.

## Artifacts

### `recall10_real100k_pqg8_n128_w500_p40_48_56_64_80_96.log`

- Lane: Task 31 M5 real 100k n128 w500 nprobe recall@10 sweep.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 40,48,56,64,80,96 --rerank-width 500 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k10.json --log-output review/30174-task31-m5-100k-n128-nprobe-sweep/artifacts/recall10_real100k_pqg8_n128_w500_p40_48_56_64_80_96.log`
- Key result:
  - `p40 recall@10=0.9760`
  - `p48 recall@10=0.9820`
  - `p56 recall@10=0.9860`
  - `p64 recall@10=0.9890`
  - `p80 recall@10=0.9960`
  - `p96 recall@10=0.9980`

### `latency_real100k_pqg8_n128_w500_p40_48_56_64_80_96.log`

- Lane: Task 31 M5 real 100k n128 w500 nprobe latency sweep.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench latency --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 40,48,56,64,80,96 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30174-task31-m5-100k-n128-nprobe-sweep/artifacts/latency_real100k_pqg8_n128_w500_p40_48_56_64_80_96.log`
- Key result:
  - `p40 p50=5.78 ms p95=6.33 ms`
  - `p48 p50=6.53 ms p95=7.12 ms`
  - `p56 p50=7.18 ms p95=7.85 ms`
  - `p64 p50=7.99 ms p95=8.56 ms`
  - `p80 p50=9.33 ms p95=10.1 ms`
  - `p96 p50=10.9 ms p95=11.6 ms`
  - `memory_samples=0` for all rows.

### `recall100_real100k_pqg8_n128_w500_p80_96.log`

- Lane: Task 31 M5 real 100k n128 w500 high-probe recall@100 spot check.
- Command:
  `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_real100k_pqg8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 80,96 --rerank-width 500 --force-index --truth-cache-file review/30173-task31-m5-pqg8-100k-n128-w500-baseline/artifacts/truth_real100k_n128_k100.json --log-output review/30174-task31-m5-100k-n128-nprobe-sweep/artifacts/recall100_real100k_pqg8_n128_w500_p80_96.log`
- Key result:
  - `p80 recall@100=0.9639`, `ndcg@100=0.9988`
  - `p96 recall@100=0.9676`, `ndcg@100=0.9991`
