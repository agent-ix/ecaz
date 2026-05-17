# Request: Extract Staged Scan Tuple Production Helper

Commit: `fefdfb8`

Summary:
- pull the top-level `amgettuple` staged execution flow in `src/am/scan.rs` behind one helper: `produce_next_scan_heap_tid`
- the helper now owns the visible tuple-production contract:
  - drain pending heap tids from the current result
  - materialize the next result if needed
  - drain the newly materialized result
- `amgettuple` now just validates scan state, delegates tuple production, and clears order-by output on false returns

Please review:
- whether this is the right visible-tuple seam for the current staged executor
- whether the helper hides any path-specific behavior that still matters before ordered traversal fully replaces the staged bootstrap/linear flow
- whether this leaves `amgettuple` in a cleaner position for the next search-owned execution slice
