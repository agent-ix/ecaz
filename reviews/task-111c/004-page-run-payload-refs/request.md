# Review Request: Task 111c Page-Run Payload Refs

## Scope

This packet implements the concrete code lever requested in packet 002 feedback:
derive page-scatter payload refs by contiguous payload page run, then accumulate
those refs into the existing cross-page TurboQuant batch. This avoids the prior
per-posting payload `single_page_slice` lookup while preserving cross-page flush
width.

Code under review:

- `f642231d6416f013d89bc4c93a3a1cd7800aae49` (`Task 111c: derive scatter payload refs by page run`)

Changed files:

- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/scan.rs`

## What Changed

- `IvfColumnarFrozenListPinnedPages::payload_page_runs` exposes contiguous
  borrowed payload runs with logical posting start and count.
- The 111c scatter scan path now iterates those runs and pushes payload refs from
  each run slice instead of resolving `payload(index)` per posting.
- The scratch still accumulates refs across page boundaries, so dense coalesced
  flush count stays aligned with the copy fallback (`109` in the 50k TQ A/B).

## Validation

Artifacts are under `reviews/task-111c/004-page-run-payload-refs/artifacts/`.

- `cargo-pgrx-test-pg18-page-scatter-equivalence.log`
  - `test tests::pg_test_ec_ivf_columnar_page_scatter_matches_copy_scan ... ok`
  - `1 passed; 0 failed; 2130 filtered out`
- `cargo-build-release-pg18.log`
  - `Finished release profile [optimized] target(s) in 5m 41s`
- `suite-status-r2.log`
  - `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

## EXPLAIN A/B Result

Fixture: local PG18, database `task111b_columnar_bench_r2`, isolated Task 111b
50k TQ columnar index, nprobe 32, rerank off, shared hits warm and reads zero.

| Cell | Logical bytes copied | Payload bytes borrowed | Dense payload copied | Dense flushes | Approx scan us | Exec ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Page scatter, page-run refs | 0 | 18,358,272 | 0 | 109 | 30,141 | 34.536 |
| Copy fallback same head | 18,887,163 | 0 | 18,358,272 | 109 | 18,986 | 23.199 |
| Page scatter packet 003 | 0 | 18,358,272 | 0 | 109 | 31,649 | 35.775 |

Interpretation:

- The requested lever is implemented and correct.
- It is a small improvement over packet 003, not a strategic win.
- Scatter still loses to copy fallback; the gate from packet 002/003 still
  stands before any fan-out across codecs/ISAs.

## Review Focus

- Is `payload_page_runs` the right reader boundary for this locality lever?
- Does the scan loop preserve ordering, live-tid budget, delete skipping, and
  cross-page batch accumulation correctly?
- Do the benchmark results support recording this as an exhausted/insufficient
  lever rather than continuing scatter fan-out?
