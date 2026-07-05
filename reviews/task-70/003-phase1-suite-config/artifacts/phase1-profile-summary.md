# Task 70 Phase 1 Profile Summary

- Head SHA: `26cc6d9de` plus packet-local suite update.
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/003-phase1-suite-config/`
- Timestamp: `2026-05-31T20:01:00Z`
- Lane / fixture / storage format / rerank mode: M5 local real10K DBpedia; `ec_diskann`; `pq_fastscan`; `rerank_budget=64`; `top_k=10`.
- Surface: isolated `task70_phase1_real10k_diskann` corpus/query/index tables.
- Data files: `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_corpus.tsv`, `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_queries.tsv`, and `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_manifest.json`; these are the staged real10K DBpedia inputs present in this checkout.
- Command: `./target/debug/ecaz bench suite run --config reviews/task-70/003-phase1-suite-config/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/003-phase1-suite-config/artifacts/suite-manifest.json --results-output reviews/task-70/003-phase1-suite-config/artifacts/results.jsonl --log-file reviews/task-70/003-phase1-suite-config/artifacts/suite-run.log`

## Recall And Latency

| list_size | recall@10 | mean q-time | latency mean | latency p50 | latency p95 | latency p99 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 0.9965 | 0.63 ms | 0.65 ms | 0.64 ms | 0.75 ms | 0.81 ms |
| 200 | 0.9975 | 0.84 ms | 0.96 ms | 0.95 ms | 1.18 ms | 1.30 ms |

## pgvectorscale Comparison

| engine | L/search_list | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ec_diskann` | 64 | 0.9965 | 0.64 ms | 0.63 ms | 0.77 ms | 0.89 ms |
| `pgvectorscale` | 64 | 0.9955 | 0.60 ms | 0.60 ms | 0.72 ms | 0.92 ms |
| `ec_diskann` | 200 | 0.9975 | 0.83 ms | 0.83 ms | 0.99 ms | 1.07 ms |
| `pgvectorscale` | 200 | 1.0000 | 1.13 ms | 1.12 ms | 1.38 ms | 1.46 ms |

`ec_diskann` index size: 4,939,776 bytes. `pgvectorscale` comparison index size: 5,136,384 bytes.

## Scan Phase Split

The profile rows are 200 `ec_diskann_scan_profile` NOTICEs per L value from suite raw steps:

- `profile-notices-diskann-real10k-l64.log`
- `profile-notices-diskann-real10k-l200.log`

| list_size | phase | mean_us | p50_us | share_of_profiled_total |
| ---: | --- | ---: | ---: | ---: |
| 64 | frontier maintenance | 269.62 | 268.5 | 72.94% |
| 64 | exact heap rerank | 80.71 | 73.0 | 21.83% |
| 64 | setup | 12.48 | 12.0 | 3.38% |
| 64 | heap prefetch | 3.85 | 4.0 | 1.04% |
| 64 | graph read/decode | 2.20 | 0.0 | 0.60% |
| 64 | binary sidecar prefilter | 0.07 | 0.0 | 0.02% |
| 64 | result expansion | 0.01 | 0.0 | 0.00% |
| 200 | frontier maintenance | 553.04 | 554.5 | 83.61% |
| 200 | exact heap rerank | 87.29 | 81.0 | 13.20% |
| 200 | setup | 13.71 | 13.0 | 2.07% |
| 200 | heap prefetch | 4.09 | 4.0 | 0.62% |
| 200 | graph read/decode | 2.60 | 0.0 | 0.39% |
| 200 | binary sidecar prefilter | 0.00 | 0.0 | 0.00% |
| 200 | result expansion | 0.01 | 0.0 | 0.00% |

Mean profiled total: 369.68 us at L=64; 661.41 us at L=200.

Mean graph/prefilter visits: 758.37 at L=64; 1,585.36 at L=200. Rerank and result counts are fixed at 64 in both runs.

## Ranked P0 Slices

1. Frontier / candidate management: P0. It accounts for 72.94% of profiled time at L=64 and 83.61% at L=200. Best-case cap is roughly the measured share; realistic first slice should target heap push/pop, visited-bitset, and bounded candidate handling.
2. Exact heap rerank fetch/detoast: P0. It accounts for 21.83% at L=64 and 13.20% at L=200. Best-case cap is the measured share; first pass should inspect whether rerank staging or tighter heap fetch scheduling reduces this without changing recall.
3. Per-scan setup / scratch reuse: P1 unless required by the frontier slice. Setup is 3.38% at L=64 and 2.07% at L=200, below the task's approximate 5% P0 threshold.
4. Graph read/decode cache: shelve for now. Graph read/decode is under 1% in this warmed local run; repeated page/tuple decoding is not a P0 cause on this fixture.
5. Binary sidecar prefilter tuning: shelve for now. The persisted binary sidecar path is active, but measured prefilter accounting is effectively zero in this profile.
6. Result materialization: shelve. Result expansion is effectively zero.

## Notes

- The raw profile SQL uses `LOAD 'ecaz'` before setting `ec_diskann.scan_profile_notice`; otherwise a placeholder custom GUC can be set before the extension library registers the real GUC and the AM does not observe the switch.
- `bench latency --session-guc ec_diskann.scan_profile_notice=on` records latency rows but does not surface per-query NOTICE payloads through its tokio-postgres path, so the suite includes raw `dev sql` steps for the phase split.
