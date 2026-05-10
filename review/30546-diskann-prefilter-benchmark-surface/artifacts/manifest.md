# Artifact Manifest

Packet: `30546-diskann-prefilter-benchmark-surface`

Head SHA: `1bb2b96610a4d7bec7c6eb0af342a84bf42931b5`

Timestamp: `2026-05-10T19:05:24Z`

Environment:

- Host lane: M5 Mac, 64GB RAM
- PostgreSQL: PG18 local pgrx scratch cluster on `/Users/peter/.pgrx`, port `28818`
- ecaz install: `./target/debug/ecaz dev install ecaz-pg-test --pg 18`
- Cluster restart: `./target/debug/ecaz dev scratch restart --pg 18`

Suite:

- Config: `crates/ecaz-cli/suites/profile-diskann-prefilter-real10k.json`
- Config SHA256: `b50d095fe225c4c06a3c2b49a39762d47c11df62a602fc7bc20d2d58c41df8f4`
- Command:

```sh
./target/debug/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/profile-diskann-prefilter-real10k.json
```

Common fixture:

- Corpus: `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_corpus.tsv`
- Queries: `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_queries.tsv`
- Rows: `10000`
- Queries limit: `200`
- k: `10`
- Isolated one-index-per-table surface: yes, prefix `profile_r10k_dann_pf`
- DiskANN build reloptions: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`
- Apples-apples scan settings: `ec_diskann.list_size == ec_diskann.rerank_budget`; pgvectorscale `diskann.query_search_list_size == diskann.query_rescore`

Artifacts:

- `suite-manifest.json`
  - Suite runner manifest for the completed benchmark.
  - Key result: `completed 8, failed 0, skipped 0`.
- `results.jsonl`
  - Normalized parsed result rows from suite report.
  - Key result lines include binary-sidecar recall and pgvectorscale comparison rows.
- `load-diskann-real10k.log`
  - Load/build log.
  - Key lines: `built profile_r10k_dann_pf_idx in 7.15s`; `completed prefix profile_r10k_dann_pf in 24.17s`.
- `recall-diskann-binary-real10k.log`
  - Lane: ec_diskann binary sidecar, real10k, matched rerank, 200 queries.
  - Key rows: `64 recall@k=0.9965 mean q-time=2.27 ms`; `800 recall@k=1.0000 mean q-time=16.32 ms`.
- `latency-diskann-binary-real10k.log`
  - Lane: ec_diskann binary sidecar, 500 latency iterations.
  - Key rows: `64 p50=2.17 ms`; `800 p50=15.7 ms`.
- `recall-diskann-grouped-real10k.log`
  - Lane: ec_diskann grouped PQ, real10k, matched rerank, 200 queries.
  - Key rows: `64 recall@k=0.9320 mean q-time=2.24 ms`; `800 recall@k=0.9990 mean q-time=15.79 ms`.
- `latency-diskann-grouped-real10k.log`
  - Lane: ec_diskann grouped PQ, 500 latency iterations.
  - Key rows: `64 p50=2.15 ms`; `800 p50=15.6 ms`.
- `compare-vectorscale-binary-real10k.log`
  - Lane: ec_diskann binary sidecar vs pgvectorscale DiskANN/SBQ.
  - Key rows: `64 ec_diskann recall@k=0.9965 p50=2.15 ms`; `64 pgvectorscale recall@k=0.9955 p50=0.59 ms`; `800 ec_diskann recall@k=1.0000 p50=15.4 ms`; `800 pgvectorscale recall@k=1.0000 p50=3.76 ms`.
  - pgvectorscale index size: `5136384 bytes`.
- `compare-vectorscale-grouped-real10k.log`
  - Lane: ec_diskann grouped PQ vs pgvectorscale DiskANN/SBQ.
  - Key rows: `64 ec_diskann recall@k=0.9320 p50=2.14 ms`; `64 pgvectorscale recall@k=0.9955 p50=0.60 ms`; `800 ec_diskann recall@k=0.9990 p50=15.2 ms`; `800 pgvectorscale recall@k=1.0000 p50=3.73 ms`.
- `storage-diskann-prefilter-real10k.log`
  - Storage footprint for `profile_r10k_dann_pf`.
  - Key rows: ec_diskann index `4.7 MiB`, `494.0 B/row`; table total `164.5 MiB`.
- `truth-real10k-k10.json`
  - Exact top-k truth cache for the 200-query real10k fixture.
