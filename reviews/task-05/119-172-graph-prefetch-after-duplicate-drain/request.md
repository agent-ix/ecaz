# Request: Keep Graph Result Prefetched After Duplicate Drain

Commit: `7f70ca7`

Summary:
- keep the graph lane prefetched after the last duplicate heap TID for the current result is emitted
- clear the just-finished current-result slot and materialize the next graph result immediately when one exists
- update debug and pg-test contracts so `current_result` represents the next ready graph result after duplicate drain completes

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/lib.rs`

Please review:
- whether clearing `current_result` and prefilling the next graph result immediately after the last duplicate drain is the right A3 boundary
- whether the new graph-only post-emit advancement preserves linear fallback behavior and scan exhaustion semantics
- whether the updated debug/pg-test surface now reflects the intended contract that graph traversal stays prefetched across `amgettuple` calls
