# Task 111c Packet 003 Summary

This checkpoint removes one allocation per borrowed columnar posting by decoding pinned-page heap TIDs directly into `IvfBorrowedPostingScratch`.

## Results

| Cell | Logical bytes copied | Payload bytes borrowed | Dense payload copied | Approx scan us | Exec ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Page scatter after heap-TID allocation removal | 0 | 18,358,272 | 0 | 31,649 | 35.775 |
| Copy fallback at same head | 18,887,163 | 0 | 18,358,272 | 16,589 | 20.720 |
| Page scatter packet 002 r3 baseline | 0 | 18,358,272 | 0 | 46,983 | 51.210 |

The change is a meaningful partial improvement: about 33% lower page-scatter approximate scan time versus packet 002 r3. It is still not the Task 111c latency win because copy fallback remains about 1.9x faster in this EXPLAIN A/B.

## Validation

- `cargo pgrx test pg18 test_ec_ivf_columnar_page_scatter_matches_copy_scan`: passed.
- `cargo build --release --no-default-features --features pg18`: passed.
- `ecaz bench suite run`: 2 completed, 0 failed, 0 stale.
