# Request: Direct Staged Selection In Tuple Production

Commit: `0d5324b`

Summary:
- remove the extra `materialize_next_scan_result` helper from `src/am/scan.rs`
- make `produce_next_scan_heap_tid` work directly in terms of:
  - emit any pending duplicate heap tid
  - select the next staged result
  - materialize it into result state
  - emit the first visible heap tid

Please review:
- whether the direct `select_next_scan_result` use in tuple production leaves the staged executor easier to reason about without changing behavior
- whether any useful boundary was lost by removing the extra boolean materialization helper
- whether this is a good base for the next step toward a single ordered result-production loop
