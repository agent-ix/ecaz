# Task 70 Packet 005 Artifact Manifest

- Head SHA: `9bbef9ecf718bab30ef543b5f21d4728267136d0`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/005-retained-frontier-heap/`
- Timestamp: `2026-05-31T19:33:01Z`
- Lane: M5 local PG18 real10K
- Fixture: `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_{corpus,queries,manifest}.tsv/json`
- Storage format: `ec_diskann` / `pq_fastscan`
- Rerank mode: index `rerank_budget=64`, `top_k=10`
- Isolation: one-index-per-table prefix `task70_005_diskann`

## Artifacts

| artifact | command | key result |
| --- | --- | --- |
| `suite.json` | checked-in `ecaz bench suite` config | Drives load, recall, latency, profile NOTICE, EXPLAIN, and pgvectorscale compare. |
| `suite-dry-run.log`, `suite-dry-run-manifest.json` | `./target/debug/ecaz bench suite run --config reviews/task-70/005-retained-frontier-heap/artifacts/suite.json --dry-run --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/005-retained-frontier-heap/artifacts/suite-dry-run-manifest.json --log-file reviews/task-70/005-retained-frontier-heap/artifacts/suite-dry-run.log` | Dry run passed and all steps target packet-local artifacts. |
| `cargo-test-diskann-scan.log` | `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` | 18 passed; 0 failed. |
| `cargo-check-pg18.log` | `cargo check --all-targets --no-default-features --features pg18` | Finished successfully. |
| `install-ecaz-pg-test.log` | `./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file reviews/task-70/005-retained-frontier-heap/artifacts/install-ecaz-pg-test.log` | Installed PG18 backend with sha256 `8277778f1c67daacfae0fd53e53451fae800a1054c3e0a91e74f0bb2e51c2aa3`. |
| `load-diskann-real10k.log` | suite load step | Built `task70_005_diskann_idx` in `7.13s`; completed prefix in `25.80s`. |
| `recall-diskann-real10k-l64-l200.log` | suite recall step | L=64 recall@10 `0.9965`, mean q-time `0.62 ms`; L=200 recall@10 `0.9975`, mean q-time `0.83 ms`. |
| `latency-diskann-real10k-l64-l200-profiled.log` | suite latency step with `ec_diskann.scan_profile_notice=on` | L=64 mean `0.64 ms`, p95 `0.73 ms`; L=200 mean `0.90 ms`, p95 `1.10 ms`. |
| `profile-notices-diskann-real10k-l64.sql`, `profile-notices-diskann-real10k-l64.log` | suite raw SQL profile step | 200 profile NOTICE rows; total mean `366.32 us`, frontier mean `261.37 us`, exact rerank mean `84.58 us`. |
| `profile-notices-diskann-real10k-l200.sql`, `profile-notices-diskann-real10k-l200.log` | suite raw SQL profile step | 200 profile NOTICE rows; total mean `641.93 us`, frontier mean `531.88 us`, exact rerank mean `88.69 us`. |
| `explain-diskann-real10k-l64.sql`, `explain-diskann-real10k-l64.log` | suite explain step | Planner gate live; effective list size `64`; index scan on `task70_005_diskann_idx`. |
| `explain-diskann-real10k-l200.sql`, `explain-diskann-real10k-l200.log` | suite explain step | Planner gate live; effective list size `200`; index scan on `task70_005_diskann_idx`. |
| `compare-vectorscale-real10k-l64-l200.log` | suite pgvectorscale compare step | L=64 `ec_diskann` mean `0.63 ms` vs pgvectorscale `0.61 ms`; L=200 `ec_diskann` mean `0.80 ms` vs pgvectorscale `1.16 ms`. |
| `results.jsonl` | suite `--results-output` | Normalized metrics for load, recall, latency, EXPLAIN, and compare. |
| `phase2-retained-frontier-summary.md` | manual packet summary from packet-local logs | Recall preserved; performance is neutral/marginal, so frontier P0 remains open. |

The loader warning about manifest prefix mismatch is expected for task-local isolated prefixes and was allowed by `allow_manifest_mismatch=true`.
