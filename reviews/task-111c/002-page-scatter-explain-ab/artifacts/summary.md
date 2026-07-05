# Task 111c Packet 002 Summary

This checkpoint closes the packet 001 reviewer gap for Task 111c AC#2 and records a current-head EXPLAIN A/B for the TQ page-scatter reference path.

## Results

| Cell | Logical bytes copied | Payload bytes borrowed | Dense payload copied | Approx scan us | Exec ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| TQ columnar page scatter | 0 | 18,358,272 | 0 | 46,983 | 51.210 |
| TQ columnar copy fallback | 18,887,163 | 0 | 18,358,272 | 17,244 | 21.379 |

The good news: the page-scatter path is real zero-copy for TQ payload bytes and the new PG18 equivalence test proves exact output and score-bit equality against the copy fallback.

The bad news: this reference path regresses latency in the 50k TQ EXPLAIN A/B. The likely next optimization target is the current per-posting borrowed-slice/reference path and page-local metadata/address generation overhead; copy removal alone is not enough in this implementation.

## Validation

- `cargo pgrx test pg18 test_ec_ivf_columnar_page_scatter_matches_copy_scan`: passed.
- `cargo pgrx test pg18 test_ec_ivf_columnar_frozen_lists_scan_insert_vacuum`: passed.
- `ecaz bench suite run` r3: 2 completed, 0 failed, 0 stale.
