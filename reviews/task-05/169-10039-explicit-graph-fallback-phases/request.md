# Review Request: Explicit Graph And Fallback Phases

Commit: `882e193`

Scope:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`

Summary:
- make the live scan state machine explicit about `GraphTraversal` versus `LinearFallback` rather
  than routing through the old bootstrap/linear naming and deciding fallback late inside
  `select_next_scan_result(...)`
- decide the fallback shell at `amrescan` time: if entry seeding fails to produce any graph
  traversal candidates, the scan enters `LinearFallback`; otherwise it stays in `GraphTraversal`
  and runs to exhaustion without a later fallback branch
- keep the debug/test surface aligned with that state split so review and pg-test scaffolding
  observe the same runtime contract as production code

Please review:
- whether choosing `LinearFallback` at `amrescan` time is the right explicit boundary for A3
- whether `GraphTraversal` now correctly owns seeded scans through exhaustion without any hidden
  fallback path remaining
- whether the renamed phase semantics in `scan_debug.rs` accurately reflect the live runtime
  contract
