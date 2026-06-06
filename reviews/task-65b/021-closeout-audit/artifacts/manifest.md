# Task 65b packet 021 artifact manifest

- Task bucket: `reviews/task-65b/021-closeout-audit`
- Head SHA under validation: `2b3a93b6cd12738f5710ad5a99a06fb3c2e0a659`
- Timestamp: `Sat Jun  6 13:40:36 UTC 2026`
- Lane: local PG18 / pgrx home `/Users/peter/.pgrx`
- Fixture: real10k host-core extension used
  `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_corpus.tsv` and
  `data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_queries.tsv`
- Storage format: `pq_fastscan`
- Rerank mode: not applicable to load-only host-core extension
- Review evidence logging: packet-local files only; no `tee` or shell
  redirection was used for these final logs.

## Formatting

- Artifact: `cargo-fmt-check.log`
- Command:
  `script -q reviews/task-65b/021-closeout-audit/artifacts/cargo-fmt-check.log cargo fmt --check`
- Result: passed. The log includes stable-rustfmt warnings for unstable
  `imports_granularity` and `group_imports` settings, with no formatting diff.

## Focused build tests

- Artifact: `cargo-test-build-task65b.log`
- Command:
  `script -q reviews/task-65b/021-closeout-audit/artifacts/cargo-test-build-task65b.log cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::build::tests::task65b`
- Result: `6 passed; 0 failed; 0 ignored; 0 measured; 1970 filtered out`.

## Default option test

- Artifact: `cargo-test-options-default.log`
- Command:
  `script -q reviews/task-65b/021-closeout-audit/artifacts/cargo-test-options-default.log cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::options::tests::diskann_default_options_include_scan_runtime_defaults`
- Result: `1 passed; 0 failed; 0 ignored; 0 measured; 1975 filtered out`.

## Full ec_diskann lib filter

- Artifact: `cargo-test-ec-diskann-single-thread.log`
- Command:
  `script -q reviews/task-65b/021-closeout-audit/artifacts/cargo-test-ec-diskann-single-thread.log cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann -- --test-threads=1`
- Result: `199 passed; 0 failed; 0 ignored; 0 measured; 1777 filtered out; finished in 51.00s`.
- Note: this broad filter was run single-threaded because the pgrx-backed tests
  mutate process-global extension state.

## Release extension reinstall

- Artifact: `install-release-after-pg-tests.log`
- Command:
  `./target/debug/ecaz --log-file reviews/task-65b/021-closeout-audit/artifacts/install-release-after-pg-tests.log dev install ecaz-pg-test --pg 18`
- Result: backend artifact assertion passed; installed backend
  `/opt/homebrew/lib/postgresql@18/ecaz.dylib`; sha256
  `b206d0568414b689d5546103fa19d07ec533023f4b6c69b2e88a0af95452d097`.

## Host-core default worker extension

- Suite config: `real10k-host-core-scaling-suite.json`
- Artifacts:
  - `real10k-host-core-scaling-release-manifest.json`
  - `real10k-host-core-scaling-release-results.jsonl`
  - `real10k-host-core-scaling-release-run.log`
  - `drop-real10k-host-core-scaling.log`
  - `load-real10k-w12-default-r32-l100.log`
  - `load-real10k-w18-default-r32-l100.log`
- Cleanup command:
  `./target/debug/ecaz --log-file reviews/task-65b/021-closeout-audit/artifacts/drop-real10k-host-core-scaling.log dev sql ecaz-pg-test --pg 18 --dbname tqvector_bench --execute "DROP TABLE IF EXISTS t65b_real10k_w12_default_l100_corpus CASCADE; DROP TABLE IF EXISTS t65b_real10k_w12_default_l100_queries CASCADE; DROP TABLE IF EXISTS t65b_real10k_w18_default_l100_corpus CASCADE; DROP TABLE IF EXISTS t65b_real10k_w18_default_l100_queries CASCADE;"`
- Suite command:
  `./target/debug/ecaz bench suite --config reviews/task-65b/021-closeout-audit/real10k-host-core-scaling-suite.json --artifact-dir reviews/task-65b/021-closeout-audit/artifacts --manifest-name real10k-host-core-scaling-release-manifest.json --results-name real10k-host-core-scaling-release-results.jsonl --log-file reviews/task-65b/021-closeout-audit/artifacts/real10k-host-core-scaling-release-run.log`
- Surface: isolated one-index-per-table prefixes
  `t65b_real10k_w12_default_l100` and
  `t65b_real10k_w18_default_l100`.
- Reloptions: `parallel_workers` only; default `parallel_build_batch_size=704`
  is capped to effective batch 64 for this 10k build.
- Key result lines:
  - w12: `build_index=0.924770s`, backend `total_ms=920`,
    `parallel_requested_workers=12`, `parallel_effective_workers=12`,
    `parallel_batch_size=64`, `parallel_epochs=157`,
    `parallel_proposal_ms=388`, `parallel_reducer_ms=183`.
  - w18: `build_index=0.835750s`, backend `total_ms=832`,
    `parallel_requested_workers=18`, `parallel_effective_workers=18`,
    `parallel_batch_size=64`, `parallel_epochs=157`,
    `parallel_proposal_ms=347`, `parallel_reducer_ms=149`.
