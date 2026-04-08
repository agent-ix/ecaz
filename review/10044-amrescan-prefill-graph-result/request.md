# Request: Prefill First Graph Result At Rescan

Commit: `be9990e`

Summary:
- make seeded graph-first scans materialize the first ordered result during `amrescan`
- keep linear fallback unchanged for unseeded scans
- update debug and pg-test contracts to treat the current result as prefilled graph runtime state

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/lib.rs`

Please review:
- whether prefilling the first graph result during `amrescan` is the right next A3 boundary toward the FR-009 “load results in rescan, drain in amgettuple” contract
- whether the new `prefill_graph_traversal_result(...)` guard conditions preserve current duplicate-drain and fallback behavior
- whether the updated debug/pg-test surfaces capture the intended runtime distinction between the prefilled current result and the remaining visible frontier
