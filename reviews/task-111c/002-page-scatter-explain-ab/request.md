# Review Request: Task 111c Page-Scatter Equivalence + EXPLAIN A/B

## Scope

This packet follows up on Task 111c packet 001 feedback:

- adds a PG18 equivalence test for `ec_ivf.columnar_page_scatter=on` vs `off`;
- exposes `Columnar Payload Bytes Borrowed` through the debug counter snapshot so tests can assert the selected path;
- records a current-head EXPLAIN A/B for the TQ columnar page-scatter reference path vs the 111b copy fallback.

Code under review:

- `77881a3e2e3e3c7bca6049e735c3e7804723c47a` (`Task 111c: add page scatter equivalence test`)
- carry-forward implementation/counter commits from packet 001 and the counter follow-up: `11b145d2d`, `75538c078`

## What Changed

- `src/am/ec_ivf/scan.rs`
  - adds `columnar_payload_bytes_borrowed` to `EcIvfGettupleCounterDebugSnapshot`.
- `src/tests/ec_ivf.rs`
  - pins the existing columnar scan/insert/vacuum copy-byte test to `ec_ivf.columnar_page_scatter=off`;
  - adds `test_ec_ivf_columnar_page_scatter_matches_copy_scan`, a multi-page 512-dimensional TQ columnar fixture that compares copy fallback vs page scatter and asserts exact `(block, offset, score.to_bits())` equality.

## Validation

Artifacts are under `reviews/task-111c/002-page-scatter-explain-ab/artifacts/`.

- `cargo-pgrx-test-pg18-columnar-page-scatter-equivalence.log`
  - `test tests::pg_test_ec_ivf_columnar_page_scatter_matches_copy_scan ... ok`
  - `1 passed; 0 failed; 2130 filtered out`
- `cargo-pgrx-test-pg18-columnar-scan-vacuum-fallback.log`
  - `test tests::pg_test_ec_ivf_columnar_frozen_lists_scan_insert_vacuum ... ok`
  - `1 passed; 0 failed; 2130 filtered out`
- `suite-status-r3.log`
  - `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

`cargo fmt --check` was attempted but is not clean repo-wide because of unrelated pre-existing formatting drift in CLI/quant files; no unrelated formatting was changed in this checkpoint.

## EXPLAIN A/B Result

Fixture: local PG18, database `task111b_columnar_bench_r2`, isolated Task 111b 50k TQ columnar index, nprobe 32, rerank off, shared hits warm and reads zero in both cells.

| Cell | Logical bytes copied | Payload bytes borrowed | Dense payload copied | Approx scan us | Exec ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Page scatter (`columnar_page_scatter=on`) | 0 | 18,358,272 | 0 | 46,983 | 51.210 |
| Copy fallback (`columnar_page_scatter=off`) | 18,887,163 | 0 | 18,358,272 | 17,244 | 21.379 |

Interpretation:

- Correctness gap from packet 001 is closed: the new PG18 test proves exact output and score-bit equality against the copy fallback.
- Counter gap is closed: page scatter reports borrowed payload bytes and zero copied payload bytes in EXPLAIN.
- This is not a latency win yet. The reference scatter path is slower than copy fallback in the r3 EXPLAIN A/B, so the next 111c work should focus on reducing per-posting borrowed-slice/reference overhead and then rerunning the full dense-a/row/columnar-scatter benchmark gate.

## Review Focus

- Is the equivalence fixture strong enough for the current TQ reference path?
- Are the debug counters now sufficient to distinguish copy fallback from page-scatter borrowing?
- Any concerns with keeping the existing scan/vacuum test pinned to the copy fallback while the new test owns scatter equivalence?
