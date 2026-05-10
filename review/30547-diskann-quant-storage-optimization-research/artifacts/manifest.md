# Manifest

- Head SHA: `8c9680bd5feaad768fb1f96a86aa239c22ce1e33`
- Packet/topic: `30547-diskann-quant-storage-optimization-research`
- Timestamp: `2026-05-10T19:10:11Z`
- Source suite: `crates/ecaz-cli/suites/profile-diskann-prefilter-real10k.json`
- Command:

```text
./target/debug/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/profile-diskann-prefilter-real10k.json
```

## Artifacts

### `results.jsonl`

- Head SHA: `8c9680bd5feaad768fb1f96a86aa239c22ce1e33`
- Lane / fixture: PG18 M5 real10k, 10k corpus rows, 200 recall queries, k=10, 500 latency queries
- Storage format: `ec_diskann` default payload with persisted binary sidecar and grouped-PQ search code
- Rerank mode: `list_size == ec_diskann.rerank_budget`; pgvectorscale `query_search_list_size == query_rescore`
- Isolated/shared: shared suite prefix with one index per engine surface
- Key cited lines:
  - binary sidecar `64`: `recall@k=0.9965`, `p50=2.15 ms`; pgvectorscale `recall@k=0.9955`, `p50=0.59 ms`
  - binary sidecar `200`: `recall@k=0.9990`, `p50=4.63 ms`; pgvectorscale `recall@k=1.0000`, `p50=1.14 ms`
  - binary sidecar `800`: `recall@k=1.0000`, `p50=15.4 ms`; pgvectorscale `recall@k=1.0000`, `p50=3.76 ms`
  - grouped-PQ `64`: `recall@k=0.9320`, `p50=2.14 ms`; pgvectorscale `recall@k=0.9955`, `p50=0.60 ms`
  - grouped-PQ `200`: `recall@k=0.9850`, `p50=4.56 ms`; pgvectorscale `recall@k=1.0000`, `p50=1.13 ms`
  - `ec_diskann` index size: `4.7 MiB`, `494.0 B/row`
  - pgvectorscale DiskANN index size: `5,136,384 bytes`

### `compare-vectorscale-binary-real10k.log`

- Head SHA: `8c9680bd5feaad768fb1f96a86aa239c22ce1e33`
- Lane / fixture: PG18 M5 real10k cross-engine compare
- Storage format: `ec_diskann` binary-sidecar prefilter vs pgvectorscale DiskANN/SBQ
- Rerank mode: matched sweep widths, `list_size == rerank_budget == query_search_list_size == query_rescore`
- Isolated/shared: shared suite prefix, cross-engine compare step
- Key cited lines:
  - `64`: `ec_diskann recall@k=0.9965 p50=2.15 ms`; `pgvectorscale recall@k=0.9955 p50=0.59 ms`
  - `200`: `ec_diskann recall@k=0.9990 p50=4.63 ms`; `pgvectorscale recall@k=1.0000 p50=1.14 ms`
  - `800`: `ec_diskann recall@k=1.0000 p50=15.4 ms`; `pgvectorscale recall@k=1.0000 p50=3.76 ms`

### `compare-vectorscale-grouped-real10k.log`

- Head SHA: `8c9680bd5feaad768fb1f96a86aa239c22ce1e33`
- Lane / fixture: PG18 M5 real10k cross-engine compare
- Storage format: `ec_diskann` grouped-PQ prefilter vs pgvectorscale DiskANN/SBQ
- Rerank mode: matched sweep widths, `list_size == rerank_budget == query_search_list_size == query_rescore`
- Isolated/shared: shared suite prefix, cross-engine compare step
- Key cited lines:
  - `64`: `ec_diskann recall@k=0.9320 p50=2.14 ms`; `pgvectorscale recall@k=0.9955 p50=0.60 ms`
  - `200`: `ec_diskann recall@k=0.9850 p50=4.56 ms`; `pgvectorscale recall@k=1.0000 p50=1.13 ms`
  - `800`: `ec_diskann recall@k=0.9990 p50=15.2 ms`; `pgvectorscale recall@k=1.0000 p50=3.73 ms`

### `storage-diskann-prefilter-real10k.log`

- Head SHA: `8c9680bd5feaad768fb1f96a86aa239c22ce1e33`
- Lane / fixture: PG18 M5 real10k storage report
- Storage format: `ec_diskann` default payload with persisted binary sidecar and grouped-PQ search code
- Rerank mode: storage-only report
- Isolated/shared: shared suite prefix
- Key cited lines:
  - `ec_diskann` index `profile_r10k_dann_pf_idx`: `4.7 MiB`, `494.0 B/row`
  - table total: `164.5 MiB`
