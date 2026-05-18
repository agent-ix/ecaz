# Review Request: Phase-Dispatched Scan Production

Commit: `4228ebd`

Scope:
- `src/am/scan.rs`

Summary:
- remove the remaining shared "select then materialize" shell from `produce_next_scan_heap_tid(...)`
- dispatch directly by explicit runtime phase so graph traversal and linear fallback each own their
  own live result-production path
- reuse the graph-phase materialization helper in production code, instead of keeping it as
  test-only scaffolding

Please review:
- whether phase-dispatched result production makes the graph-first runtime path clearer without
  changing staged behavior
- whether `materialize_next_bootstrap_frontier_result(...)` is now the right production boundary
  for graph-phase result materialization
- whether the linear fallback path still preserves duplicate-drain/current-result semantics after
  the shared selector was removed
