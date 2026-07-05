# Task 111c Packet 002 Artifact Manifest

- Task bucket: `reviews/task-111c/`
- Packet: `reviews/task-111c/002-page-scatter-explain-ab/`
- Head SHA: `77881a3e2e3e3c7bca6049e735c3e7804723c47a`
- Timestamp: 2026-06-17
- Lane: local PG18, pgrx PG 18.3, socket `/home/peter/.pgrx`
- Fixture surface: reused Task 111b packet 008 isolated one-index-per-table surface in database `task111b_columnar_bench_r2`
- Fixture cell: `task111b_008_50k_tq_columnar`, `turboquant`, 50k rows, 1536 dimensions, nlists 64, nprobe 32, rerank off

## Code And Test Artifacts

### `cargo-build-release-pg18-r3.log`

- Command: `script -q -c "cargo build --release --no-default-features --features pg18" reviews/task-111c/002-page-scatter-explain-ab/artifacts/cargo-build-release-pg18-r3.log`
- Result: `Finished release profile [optimized] target(s) in 5m 45s`

### `install-ecaz-pg18-release-r3.log`

- Command: `target/release/ecaz dev install ecaz-pg-test --pg 18 --pgrx-home /home/peter/.pgrx --log-file reviews/task-111c/002-page-scatter-explain-ab/artifacts/install-ecaz-pg18-release-r3.log`
- Result: installed backend `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- Installed backend SHA256: `109d98d186178a477442eb4d15ad21785379f663d68b35344f336b16f3950225`

### `cargo-pgrx-test-pg18-columnar-page-scatter-equivalence.log`

- Command: `script -q -c "cargo pgrx test pg18 test_ec_ivf_columnar_page_scatter_matches_copy_scan" reviews/task-111c/002-page-scatter-explain-ab/artifacts/cargo-pgrx-test-pg18-columnar-page-scatter-equivalence.log`
- Result: `test tests::pg_test_ec_ivf_columnar_page_scatter_matches_copy_scan ... ok`
- Summary: `1 passed; 0 failed; 2130 filtered out`
- Coverage: multi-page 512-dimensional TQ columnar list, `ec_ivf.columnar_page_scatter=off` vs `on`, exact `(block, offset, score.to_bits())` equality; copy path reports logical copy bytes and scatter path reports borrowed payload bytes.

### `cargo-pgrx-test-pg18-columnar-scan-vacuum-fallback.log`

- Command: `script -q -c "cargo pgrx test pg18 test_ec_ivf_columnar_frozen_lists_scan_insert_vacuum" reviews/task-111c/002-page-scatter-explain-ab/artifacts/cargo-pgrx-test-pg18-columnar-scan-vacuum-fallback.log`
- Result: `test tests::pg_test_ec_ivf_columnar_frozen_lists_scan_insert_vacuum ... ok`
- Summary: `1 passed; 0 failed; 2130 filtered out`
- Coverage: existing columnar scan/insert/vacuum fixture pinned to `ec_ivf.columnar_page_scatter=off` so its copy-byte assertions remain path-specific.

## Suite Artifacts

### `task111c-page-scatter-explain-suite.json`

- Command/config source for the two-step `ecaz bench suite run` EXPLAIN A/B.
- Steps:
  - `explain-50k-tq-columnar-page-scatter`: `ec_ivf.columnar_page_scatter=on`
  - `explain-50k-tq-columnar-copy-fallback`: `ec_ivf.columnar_page_scatter=off`

### `suite-run-r3.log`

- Command: `target/release/ecaz bench suite run --config reviews/task-111c/002-page-scatter-explain-ab/artifacts/task111c-page-scatter-explain-suite.json --artifact-dir reviews/task-111c/002-page-scatter-explain-ab/artifacts/suite-r3 --database task111b_columnar_bench_r2 --host /home/peter/.pgrx --log-file reviews/task-111c/002-page-scatter-explain-ab/artifacts/suite-run-r3.log`
- Result: completed both EXPLAIN steps and wrote `suite-r3/suite-manifest.json` plus `suite-r3/results.jsonl`.

### `suite-status-r3.log`

- Command: `target/release/ecaz bench suite status --manifest reviews/task-111c/002-page-scatter-explain-ab/artifacts/suite-r3/suite-manifest.json --database task111b_columnar_bench_r2 --host /home/peter/.pgrx --log-file reviews/task-111c/002-page-scatter-explain-ab/artifacts/suite-status-r3.log`
- Result: `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

### `suite-r3/explain-50k-tq-columnar-page-scatter.log`

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
  - `Approximate Scan Elapsed Us`: 46983
  - `Execution Time`: 51.210 ms

### `suite-r3/explain-50k-tq-columnar-copy-fallback.log`

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
  - `Approximate Scan Elapsed Us`: 17244
  - `Execution Time`: 21.379 ms

## Interpretation

- Correctness: the new PG18 equivalence test passes with exact score-bit equality between copy fallback and page scatter on a multi-page TQ columnar list.
- Counter proof: page scatter eliminates the TQ payload assembly copy in the EXPLAIN fixture (`Columnar Logical Bytes Copied = 0`, `Dense Coalesced Payload Bytes Copied = 0`) and reports 18.36 MB of borrowed payload bytes.
- Performance: this reference scatter path is slower than the copy fallback in the current EXPLAIN A/B (`46983 us` vs `17244 us` approximate scan; `51.210 ms` vs `21.379 ms` execution). This packet is therefore a correctness/counter checkpoint, not a latency-win/promote packet.
