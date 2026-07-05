# Task 70 Packet 006 Artifact Manifest

- Measured code SHA: `4f499e27910399760f8c535588a8fdab805bc1b6`
- Shelve/revert SHA: `3f3fa8bfe6f6e103e4219a8a54c1c8c879fb20fb`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/006-rerank-result-retention/`
- Timestamp: `2026-05-31T19:58:12Z`
- Lane: M5 local PG18 real10K
- Fixture: `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_{corpus,queries,manifest}.tsv/json`
- Storage format: `ec_diskann` / `pq_fastscan`
- Rerank mode: index `rerank_budget=64`, `top_k=10`
- Isolation: one-index-per-table prefix `task70_006_diskann`

## Artifacts

| artifact | command | key result |
| --- | --- | --- |
| `suite.json` | checked-in `ecaz bench suite` config | Drives load, recall, latency, profile NOTICE, EXPLAIN, and pgvectorscale compare. |
| `suite-dry-run.log`, `suite-dry-run-manifest.json` | `./target/debug/ecaz bench suite run --config reviews/task-70/006-rerank-result-retention/artifacts/suite.json --dry-run --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/006-rerank-result-retention/artifacts/suite-dry-run-manifest.json --log-file reviews/task-70/006-rerank-result-retention/artifacts/suite-dry-run.log` | Dry run passed and all steps target packet-local artifacts. |
| `cargo-test-diskann-scan.log` | `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` | 18 passed; 0 failed before measurement. |
| `cargo-check-pg18.log` | `cargo check --all-targets --no-default-features --features pg18` | Finished successfully before measurement. |
| `install-ecaz-pg-test.log` | `./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file reviews/task-70/006-rerank-result-retention/artifacts/install-ecaz-pg-test.log` | Installed measured PG18 backend with sha256 `a85a1394c6043cfe7a14268d56ec075aa5e7e64d629c6151c191c4e82844ef18`. |
| `load-diskann-real10k.log` | suite load step | Built `task70_006_diskann_idx` in `7.06s`; completed prefix in `25.63s`. |
| `recall-diskann-real10k-l64-l200.log` | suite recall step | L=64 recall@10 `0.9965`, mean q-time `0.62 ms`; L=200 recall@10 `0.9975`, mean q-time `0.83 ms`. |
| `latency-diskann-real10k-l64-l200-profiled.log` | suite latency step with `ec_diskann.scan_profile_notice=on` | L=64 mean `0.63 ms`, p95 `0.74 ms`; L=200 mean `0.96 ms`, p95 `1.23 ms`. |
| `profile-notices-diskann-real10k-l64.sql`, `profile-notices-diskann-real10k-l64.log` | suite raw SQL profile step | 200 profile NOTICE rows; total mean `386.21 us`, frontier mean `266.97 us`, exact rerank mean `97.64 us`. |
| `profile-notices-diskann-real10k-l200.sql`, `profile-notices-diskann-real10k-l200.log` | suite raw SQL profile step | 200 profile NOTICE rows; total mean `668.97 us`, frontier mean `544.10 us`, exact rerank mean `101.02 us`. |
| `explain-diskann-real10k-l64.sql`, `explain-diskann-real10k-l64.log` | suite explain step | Planner gate live; effective list size `64`; index scan on `task70_006_diskann_idx`. |
| `explain-diskann-real10k-l200.sql`, `explain-diskann-real10k-l200.log` | suite explain step | Planner gate live; effective list size `200`; index scan on `task70_006_diskann_idx`. |
| `compare-vectorscale-real10k-l64-l200.log` | suite pgvectorscale compare step | L=64 `ec_diskann` mean `0.64 ms` vs pgvectorscale `0.61 ms`; L=200 `ec_diskann` mean `0.80 ms` vs pgvectorscale `1.17 ms`. |
| `results.jsonl` | suite `--results-output` | Normalized metrics for load, recall, latency, EXPLAIN, and compare. |
| `cargo-test-after-revert.log` | `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` | 18 passed; 0 failed after revert commit `3f3fa8bfe6f6e103e4219a8a54c1c8c879fb20fb`. |
| `phase2-rerank-retention-summary.md` | manual packet summary from packet-local logs | Slice is negative and has been shelved by revert. |

The loader warning about manifest prefix mismatch is expected for task-local isolated prefixes and was allowed by `allow_manifest_mismatch=true`.
