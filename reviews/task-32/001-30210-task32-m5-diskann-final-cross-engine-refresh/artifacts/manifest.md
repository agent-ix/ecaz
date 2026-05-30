# Artifact Manifest: Task 32 M5 DiskANN Final Cross-Engine Refresh

- head SHA: `82f3cd6877d4af5156d91138feed435795343ce7`
- task bucket: `reviews/task-32`
- packet path: `reviews/task-32/001-30210-task32-m5-diskann-final-cross-engine-refresh`
- lane: Task 32 final post-M5 DiskANN cross-engine refresh
- timestamp: `2026-05-30T15:13:56Z`
- hardware: Apple M5 local development machine
- PostgreSQL: PG18 on socket `/Users/peter/.pgrx`, port `28818`
- fixture: real10K corpus from `data/task31_m5_dbpedia_staged/`
- corpus/query rows: `10000` corpus rows, `200` query rows
- corpus hash: `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`
- query hash: `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- isolation/shared-table surface: isolated one-index-per-prefix surfaces
- cache state: warm cache, matching the original Task 29d comparison surface
- suite config: `task32-m5-diskann-final-cross-engine.packet.json`
- suite manifest: `artifacts/suite-manifest.json`
- normalized results: `artifacts/results.jsonl`

## Surface

The packet refreshes the Task 29d real10K cross-engine comparison on the final
M5 DiskANN code state:

- `ec_diskann`: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`,
  `list_size=64,128,200,400,800`
- `pgvectorscale`: DiskANN comparator with `num_neighbors=32`,
  `search_list_size=100`, `max_alpha=1.2`,
  `query_search_list_size=64,128,200,400,800`
- `ec_hnsw`: `m=32`, `ef_construction=100`, `ef_search=64,128,200,400,800`
- `pgvector`: HNSW comparator with `m=32`, `ef_construction=100`,
  `ef_search=64,128,200,400,800`

## Commands

The suite was run with:

```sh
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-32/001-30210-task32-m5-diskann-final-cross-engine-refresh/task32-m5-diskann-final-cross-engine.packet.json --manifest-output reviews/task-32/001-30210-task32-m5-diskann-final-cross-engine-refresh/artifacts/suite-manifest.json --results-output reviews/task-32/001-30210-task32-m5-diskann-final-cross-engine-refresh/artifacts/results.jsonl
```

The runner executed the following packet-local steps:

- `load-diskann-real10k-final`: `corpus load --prefix task32_m5_diskann_final --profile ec_diskann ... --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2`
- `recall-diskann-real10k-final`: `bench recall --prefix task32_m5_diskann_final --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --queries-limit 200`
- `latency-diskann-real10k-final`: `bench latency --prefix task32_m5_diskann_final --profile ec_diskann --k 10 --iterations 500 --sweep 64,128,200,400,800 --sample-backend-memory --memory-sample-interval-ms 25`
- `storage-diskann-real10k-final`: `bench storage --prefix task32_m5_diskann_final`
- `compare-vectorscale-real10k-final`: `compare vectorscale --prefix task32_m5_diskann_final --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --vectorscale-num-neighbors 32 --vectorscale-build-search-list-size 100 --vectorscale-max-alpha 1.2 --rebuild`
- `load-hnsw-real10k-final`: `corpus load --prefix task32_m5_hnsw_final --profile ec_hnsw ... --m 32 --ef-construction 100`
- `recall-hnsw-real10k-final`: `bench recall --prefix task32_m5_hnsw_final --profile ec_hnsw --k 10 --sweep 64,128,200,400,800 --queries-limit 200`
- `latency-hnsw-real10k-final`: `bench latency --prefix task32_m5_hnsw_final --profile ec_hnsw --k 10 --iterations 500 --sweep 64,128,200,400,800 --sample-backend-memory --memory-sample-interval-ms 25`
- `storage-hnsw-real10k-final`: `bench storage --prefix task32_m5_hnsw_final`
- `compare-pgvector-real10k-hnsw-final`: `compare pgvector --prefix task32_m5_hnsw_final --profile ec_hnsw --k 10 --sweep 64,128,200,400,800 --pgvector-m 32 --pgvector-ef-construction 100 --rebuild`

The exact index-size follow-up was captured with:

```sh
/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --port 28818 --raw --sql "select c.relname, pg_relation_size(c.oid) as bytes from pg_class c where c.relname in ('task32_m5_diskann_final_idx','task32_m5_diskann_final_corpus_vectorscale_diskann_idx','task32_m5_hnsw_final_m32_idx','task32_m5_hnsw_final_corpus_pgvector_hnsw_idx') order by c.relname;" --log-output reviews/task-32/001-30210-task32-m5-diskann-final-cross-engine-refresh/artifacts/index-size-bytes.sql.log
```

## Artifact Index

| Artifact | Purpose |
| --- | --- |
| `suite-manifest.json` | `ecaz bench suite` execution manifest and per-step commands |
| `results.jsonl` | normalized parsed rows plus summary rows for docs consumption |
| `audit.log` | suite audit output |
| `load-diskann-real10k-final.log` | `ec_diskann` load/build status; total `9.84s` |
| `recall-diskann-real10k-final-table.log` | `ec_diskann` recall sweep |
| `latency-diskann-real10k-final-table.log` | `ec_diskann` latency sweep |
| `storage-diskann-real10k-final.log` | `ec_diskann` storage table |
| `compare-vectorscale-real10k-final.log` | `ec_diskann` vs `pgvectorscale` sweep, pgvectorscale build time and size |
| `load-hnsw-real10k-final.log` | `ec_hnsw` load/build status; total `9.86s` |
| `recall-hnsw-real10k-final-table.log` | `ec_hnsw` recall sweep |
| `latency-hnsw-real10k-final-table.log` | `ec_hnsw` latency sweep |
| `storage-hnsw-real10k-final.log` | `ec_hnsw` storage table |
| `compare-pgvector-real10k-hnsw-final.log` | `ec_hnsw` vs `pgvector` sweep, pgvector build time and size |
| `index-size-bytes.sql.log` | exact `pg_relation_size` rows for all four compared indexes |
| `truth_real10k_k10.json` | packet-local exact truth cache |

## Key Results

At the matched low tuning point:

| engine | tuning | recall@10 | mean | p50 | p95 | p99 | build | index size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ec_diskann` | `64` | `0.9965` | `2.14 ms` | `2.11 ms` | `2.38 ms` | `2.67 ms` | `9.84 s` | `4,939,776 B` |
| `pgvectorscale` | `64` | `0.9960` | `0.60 ms` | `0.59 ms` | `0.73 ms` | `0.88 ms` | `1.48 s` | `5,136,384 B` |
| `ec_hnsw` | `64` | `0.9695` | `1.35 ms` | `1.26 ms` | `1.90 ms` | `2.50 ms` | `9.86 s` | `15,130,624 B` |
| `pgvector` | `64` | `0.9980` | `0.55 ms` | `0.48 ms` | `0.96 ms` | `1.32 ms` | `6.16 s` | `81,928,192 B` |

Across the DiskANN-vs-pgvectorscale sweep:

- `ec_diskann[list_size=64]`: recall@10 `0.9965`, mean `2.14 ms`, p99 `2.67 ms`
- `pgvectorscale[query_search_list_size=64]`: recall@10 `0.9960`, mean `0.60 ms`, p99 `0.88 ms`
- `ec_diskann[list_size=200]`: recall@10 `0.9970`, mean `2.79 ms`, p99 `5.71 ms`
- `pgvectorscale[query_search_list_size=200]`: recall@10 `1.0000`, mean `1.37 ms`, p99 `3.90 ms`
- `ec_diskann[list_size=800]`: recall@10 `0.9975`, mean `2.88 ms`, p99 `3.69 ms`
- `pgvectorscale[query_search_list_size=800]`: recall@10 `1.0000`, mean `3.74 ms`, p99 `4.63 ms`

The honest local M5 conclusion is that `ec_diskann` preserves high recall on the
final code state, but `pgvectorscale` is materially faster at the matched
low-list operating point and builds much faster on this fixture. `ec_diskann`
only overtakes `pgvectorscale` at the high `800` tuning point in this warm-cache
real10K sweep.

## Instrumentation Notes

- The latency commands requested backend memory sampling, but every latency row
  has `memory_samples=0`, `rss_peak_kb=0`, and `hwm_peak_kb=0`. Treat memory HWM
  as not measured for this packet; do not publish `0 KB` as a memory claim.
- The original corpus manifest uses prefix `ec_hnsw_real_10k`; the suite used
  `--allow-manifest-mismatch` and recorded matching corpus/query hashes before
  loading the Task 32 prefixes.
- No Rust or SQL code changed in this packet. No tests were run for this
  metadata cleanup; validation is artifact consistency against the existing
  packet-local logs.
