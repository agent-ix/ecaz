# Request: Trim Legacy Bootstrap Materialization Surface

Commit: `5de9a4b`

Summary:
- remove the unused `materialize_scan_candidate_result` wrapper from `src/am/scan.rs`
- fence the remaining `materialize_next_bootstrap_frontier_result` helper to `test` / `pg_test` builds only
- keep runtime execution on the newer staged path:
  - select next staged result
  - materialize selected result
  - emit visible heap tid

Please review:
- whether any remaining debug/test-only materialization helper still looks like production API surface by accident
- whether the test-only bootstrap materialization wrapper is the right minimal seam for current pg/debug coverage
- whether the production executor surface now looks meaningfully closer to the real runtime contract
