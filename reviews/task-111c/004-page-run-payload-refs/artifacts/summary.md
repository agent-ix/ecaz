# Task 111c Packet 004 Summary

This checkpoint tries the reviewer-requested 111c lever from packet 002:
derive borrowed payload refs by contiguous payload page run, then accumulate
those refs into the existing cross-page TurboQuant batch. This removes the
per-posting payload `single_page_slice` lookup from the scatter path while
preserving the prior cross-page flush width.

## Warm EXPLAIN A/B Result

Packet-local r2 suite, local PG18, database `task111b_columnar_bench_r2`,
fixture `task111b_008_50k_tq_columnar`, nprobe 32, rerank off, shared reads 0.

| Cell | Logical bytes copied | Payload bytes borrowed | Dense payload copied | Dense flushes | Approx scan us | Exec ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Page scatter, page-run refs | 0 | 18,358,272 | 0 | 109 | 30,141 | 34.536 |
| Copy fallback same head | 18,887,163 | 0 | 18,358,272 | 109 | 18,986 | 23.199 |
| Page scatter packet 003 | 0 | 18,358,272 | 0 | 109 | 31,649 | 35.775 |
| Page scatter packet 002 r3 | 0 | 18,358,272 | 0 | 109 | 46,983 | 51.210 |

## Interpretation

- The page-run ref lever preserves exact-score equivalence and zero-copy payload
  counters.
- It is only a small latency improvement versus packet 003: approximate scan
  `31,649 -> 30,141 us`, execution `35.775 -> 34.536 ms`.
- It still trails copy fallback: `30,141 us` vs `18,986 us` approximate scan in
  the r2 run.
- The first suite run recorded a scatter outlier (`325,687 us` approximate scan,
  `337.588 ms` execution) while the copy fallback remained normal-ish
  (`24,941 us`, `30.811 ms`). The r2 run is the comparison cited above because
  both cells are warm and internally consistent.

## Validation

- `cargo pgrx test pg18 test_ec_ivf_columnar_page_scatter_matches_copy_scan`: passed.
- `cargo build --release --no-default-features --features pg18`: passed.
- `ecaz bench suite run`: r1 and r2 each completed 2 steps with 0 failed/stale.
