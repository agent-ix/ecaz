# Task 111c Packet 003 Artifact Manifest

- Task bucket: `reviews/task-111c/`
- Packet: `reviews/task-111c/003-page-scatter-heap-tid-decode/`
- Head SHA: `5e43d2c2f9d4edc99dbf8f597dd467bb0a8c965a`
- Timestamp: 2026-06-17
- Lane: local PG18, pgrx PG 18.3, socket `/home/peter/.pgrx`
- Fixture surface: reused Task 111b packet 008 isolated one-index-per-table surface in database `task111b_columnar_bench_r2`
- Fixture cell: `task111b_008_50k_tq_columnar`, `turboquant`, 50k rows, 1536 dimensions, nlists 64, nprobe 32, rerank off

## Code And Test Artifacts

### `cargo-pgrx-test-pg18-page-scatter-equivalence.log`

- Command: `script -q -c "cargo pgrx test pg18 test_ec_ivf_columnar_page_scatter_matches_copy_scan" reviews/task-111c/003-page-scatter-heap-tid-decode/artifacts/cargo-pgrx-test-pg18-page-scatter-equivalence.log`
- Result: `test tests::pg_test_ec_ivf_columnar_page_scatter_matches_copy_scan ... ok`
- Summary: `1 passed; 0 failed; 2130 filtered out`
- Coverage: multi-page TQ columnar page-scatter path remains exact-score equivalent to copy fallback after allocation removal.

### `cargo-build-release-pg18.log`

- Command: `script -q -c "cargo build --release --no-default-features --features pg18" reviews/task-111c/003-page-scatter-heap-tid-decode/artifacts/cargo-build-release-pg18.log`
- Result: `Finished release profile [optimized] target(s) in 5m 45s`

### `install-ecaz-pg18-release.log`

- Command: `target/release/ecaz dev install ecaz-pg-test --pg 18 --pgrx-home /home/peter/.pgrx --log-file reviews/task-111c/003-page-scatter-heap-tid-decode/artifacts/install-ecaz-pg18-release.log`
- Result: installed backend `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- Installed backend SHA256: `3b0ebca69578805a48320e1a0bc3648bbf6baf99a8a081d05eadf1451655ff53`

## Suite Artifacts

### `task111c-page-scatter-explain-suite.json`

- Same two-step EXPLAIN A/B shape as packet 002.
- Steps:
  - `explain-50k-tq-columnar-page-scatter`: `ec_ivf.columnar_page_scatter=on`
  - `explain-50k-tq-columnar-copy-fallback`: `ec_ivf.columnar_page_scatter=off`

### `suite-run.log`

- Command: `target/release/ecaz bench suite run --config reviews/task-111c/003-page-scatter-heap-tid-decode/artifacts/task111c-page-scatter-explain-suite.json --artifact-dir reviews/task-111c/003-page-scatter-heap-tid-decode/artifacts/suite --database task111b_columnar_bench_r2 --host /home/peter/.pgrx --log-file reviews/task-111c/003-page-scatter-heap-tid-decode/artifacts/suite-run.log`
- Result: completed both EXPLAIN steps and wrote `suite/suite-manifest.json` plus `suite/results.jsonl`.

### `suite-status.log`

- Command: `target/release/ecaz bench suite status --manifest reviews/task-111c/003-page-scatter-heap-tid-decode/artifacts/suite/suite-manifest.json --database task111b_columnar_bench_r2 --host /home/peter/.pgrx --log-file reviews/task-111c/003-page-scatter-heap-tid-decode/artifacts/suite-status.log`
- Result: `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

### `suite/explain-50k-tq-columnar-page-scatter.log`

- Lane/fixture/storage: local PG18, `task111b_columnar_bench_r2`, isolated 50k TQ columnar index, `ec_ivf.columnar_page_scatter=on`
- Key result lines:
  - `Columnar Frozen Lists Visited`: 32
  - `Columnar Postings Visited`: 23904
  - `Columnar Logical Bytes Copied`: 0
  - `Columnar Payload Bytes Borrowed`: 18358272
  - `Dense Coalesced Flushes`: 109
  - `Dense Coalesced Payload Bytes Copied`: 0
  - `Dense Coalesced Heap TID Bytes Copied`: 143424
  - `Shared Hit Blocks`: 5557
  - `Shared Read Blocks`: 0
  - `Approximate Scan Elapsed Us`: 31649
  - `Execution Time`: 35.775 ms

### `suite/explain-50k-tq-columnar-copy-fallback.log`

- Lane/fixture/storage: local PG18, `task111b_columnar_bench_r2`, isolated 50k TQ columnar index, `ec_ivf.columnar_page_scatter=off`
- Key result lines:
  - `Columnar Frozen Lists Visited`: 32
  - `Columnar Postings Visited`: 23904
  - `Columnar Logical Bytes Copied`: 18887163
  - `Columnar Payload Bytes Borrowed`: 0
  - `Dense Coalesced Flushes`: 109
  - `Dense Coalesced Payload Bytes Copied`: 18358272
  - `Dense Coalesced Heap TID Bytes Copied`: 143424
  - `Shared Hit Blocks`: 5557
  - `Shared Read Blocks`: 0
  - `Approximate Scan Elapsed Us`: 16589
  - `Execution Time`: 20.720 ms

## Interpretation

- The allocation removal preserves exact-score equivalence and zero-copy payload counters.
- Compared with packet 002 r3, page-scatter approximate scan improves from `46983 us` to `31649 us` and execution improves from `51.210 ms` to `35.775 ms`.
- The path still trails copy fallback (`31649 us` vs `16589 us` approximate scan). Remaining overhead is likely per-posting page/metadata lookup and generic borrowed payload batch construction, not heap-TID allocation alone.
