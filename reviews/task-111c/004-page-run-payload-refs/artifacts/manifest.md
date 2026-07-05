# Task 111c Packet 004 Artifact Manifest

- Task bucket: `reviews/task-111c/`
- Packet: `reviews/task-111c/004-page-run-payload-refs/`
- Head SHA: `f642231d6416f013d89bc4c93a3a1cd7800aae49`
- Timestamp: 2026-06-17
- Lane: local PG18, pgrx PG 18.3, socket `/home/peter/.pgrx`
- Fixture surface: reused Task 111b packet 008 isolated one-index-per-table surface in database `task111b_columnar_bench_r2`
- Fixture cell: `task111b_008_50k_tq_columnar`, `turboquant`, 50k rows, 1536 dimensions, nlists 64, nprobe 32, rerank off

## Code And Test Artifacts

### `cargo-pgrx-test-pg18-page-scatter-equivalence.log`

- Command: `script -q -c "cargo pgrx test pg18 test_ec_ivf_columnar_page_scatter_matches_copy_scan" reviews/task-111c/004-page-run-payload-refs/artifacts/cargo-pgrx-test-pg18-page-scatter-equivalence.log`
- Result: `test tests::pg_test_ec_ivf_columnar_page_scatter_matches_copy_scan ... ok`
- Summary: `1 passed; 0 failed; 2130 filtered out`
- Coverage: multi-page TQ columnar page-scatter path remains exact-score equivalent to copy fallback after deriving payload refs from page runs.

### `cargo-build-release-pg18.log`

- Command: `script -q -c "cargo build --release --no-default-features --features pg18" reviews/task-111c/004-page-run-payload-refs/artifacts/cargo-build-release-pg18.log`
- Result: `Finished release profile [optimized] target(s) in 5m 41s`

### `install-ecaz-pg18-release.log`

- Command: `target/release/ecaz dev install ecaz-pg-test --pg 18 --pgrx-home /home/peter/.pgrx --log-file reviews/task-111c/004-page-run-payload-refs/artifacts/install-ecaz-pg18-release.log`
- Result: installed backend `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- Installed backend SHA256: `390838e8de4d5d4eca6d7ca33e3e206319b9b299ea5d205468e169d24a8218bc`

## Suite Artifacts

### `task111c-page-run-payload-refs-suite.json`

- Two-step EXPLAIN A/B shape:
  - `explain-50k-tq-columnar-page-scatter`: `ec_ivf.columnar_page_scatter=on`
  - `explain-50k-tq-columnar-copy-fallback`: `ec_ivf.columnar_page_scatter=off`

### `suite-run.log` / `suite-status.log`

- Command: `target/release/ecaz bench suite run --config reviews/task-111c/004-page-run-payload-refs/artifacts/task111c-page-run-payload-refs-suite.json --artifact-dir reviews/task-111c/004-page-run-payload-refs/artifacts/suite --database task111b_columnar_bench_r2 --host /home/peter/.pgrx --log-file reviews/task-111c/004-page-run-payload-refs/artifacts/suite-run.log`
- Status command: `target/release/ecaz bench suite status --manifest reviews/task-111c/004-page-run-payload-refs/artifacts/suite/suite-manifest.json --database task111b_columnar_bench_r2 --host /home/peter/.pgrx --log-file reviews/task-111c/004-page-run-payload-refs/artifacts/suite-status.log`
- Result: `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- Note: first scatter cell was an outlier (`325,687 us`, `337.588 ms`), so r2 is the cited warm comparison.

### `suite-run-r2.log` / `suite-status-r2.log`

- Command: `target/release/ecaz bench suite run --config reviews/task-111c/004-page-run-payload-refs/artifacts/task111c-page-run-payload-refs-suite.json --artifact-dir reviews/task-111c/004-page-run-payload-refs/artifacts/suite-r2 --database task111b_columnar_bench_r2 --host /home/peter/.pgrx --log-file reviews/task-111c/004-page-run-payload-refs/artifacts/suite-run-r2.log`
- Status command: `target/release/ecaz bench suite status --manifest reviews/task-111c/004-page-run-payload-refs/artifacts/suite-r2/suite-manifest.json --database task111b_columnar_bench_r2 --host /home/peter/.pgrx --log-file reviews/task-111c/004-page-run-payload-refs/artifacts/suite-status-r2.log`
- Result: `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

### `suite-r2/explain-50k-tq-columnar-page-scatter.log`

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
  - `Approximate Scan Elapsed Us`: 30141
  - `Execution Time`: 34.536 ms

### `suite-r2/explain-50k-tq-columnar-copy-fallback.log`

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
  - `Approximate Scan Elapsed Us`: 18986
  - `Execution Time`: 23.199 ms
