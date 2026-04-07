# Request: Drain Pending Current-Result Heap TIDs Before Fallback Paths

Commit: `124ff6e`

Summary:
- `amgettuple` now drains any already-materialized pending heap TIDs before it attempts either bootstrap frontier adjudication or the linear fallback path.
- This makes duplicate emission from one `current_result` explicit in the runtime control flow instead of implicitly reusing `next_linear_scan_heap_tid` as a side effect.
- The slice also adds a unit regression that pins `take_pending_scan_heap_tid` as the source of current-result heap-progress state.

Files:
- `src/am/scan.rs`

Why this matters:
- The staged executor already shares one `current_result` plus pending-heap-TID drain slot across bootstrap and linear production.
- Before this slice, later duplicates from a bootstrap-materialized result were emitted only because control fell through to the linear helper, which blurred the real runtime contract.
- Making pending drain its own first-class step should be a better base for later work where result production gets more genuinely ordered and less tied to the old linear-scan helper.

Review focus:
- Whether draining pending heap TIDs before both bootstrap and linear selection is the right current runtime contract
- Whether this change cleanly separates “emit more heap tids for the current result” from “find the next result candidate”
- Whether any current path still depends on the old accidental coupling between pending duplicate drain and `next_linear_scan_heap_tid`
