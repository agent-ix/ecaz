# Request: Resolved ef_search for Bootstrap Runtime Frontier

Commit: `f12e74c`

Summary:
- switch `amrescan` bootstrap frontier sizing from the raw index reloption to the existing resolved
  planner/runtime tuning helper
- make bootstrap beam width consume the effective `ef_search` value, so non-default
  `SET tqhnsw.ef_search = ...` overrides now affect the graph-owned bootstrap traversal seam
- add a pg_test that proves a session override can narrow the runtime bootstrap frontier even when
  the index reloption is wider

Please review:
- whether using `resolve_scan_tuning(...)` in `amrescan` is the right runtime seam before ordered
  traversal is fully graph-owned
- whether the new runtime pg_test is the right contract for the current staged behavior in
  FR-009 / ADR-016
