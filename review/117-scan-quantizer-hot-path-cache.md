# Request: Cache Scan Quantizer Across Candidate Scoring

Commit: `30737c9`

Summary:
- Keeps the prepared query's `ProdQuantizer` in scan-owned state instead of reacquiring it from the global cache on every scored element.
- Frees that cached quantizer alongside the prepared query during scan teardown and rescan replacement.
- Adds a direct unit test covering the prepared-query plus quantizer lifetime pair.

Files:
- `src/am/scan.rs`

Why this matters:
- The current bootstrap scan scores every candidate through `score_scan_element_result`.
- Before this slice, that helper re-entered `ProdQuantizer::cached(...)` for every element score, paying a mutex lookup and `Arc` clone on the hot path even though `amrescan` had already fixed the quantizer shape for the whole scan.
- This keeps scan scoring aligned with the existing prepared-query cache: resolve the quantizer once at rescan time, then reuse it until `amendscan` or the next rescan.

Review focus:
- Whether the scan-owned quantizer lifetime is paired correctly with prepared-query allocation and free paths
- Whether retaining the quantizer as a raw `Arc::into_raw` / `Arc::from_raw` pointer is the right ownership shape for scan state
- Whether any remaining scoring paths still accidentally reacquire quantizer cache state per candidate
