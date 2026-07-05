# Review Request: Graph-First Rescan Exhaustion Test Contract

Commit: `3c61aaa`

Scope:
- `src/lib.rs`

Summary:
- update `pg_test_tqhnsw_gettuple_rescan_after_exhaustion_restarts_scan` to the staged A3
  graph-first contract
- keep the rescan guarantee that tuple production restarts from the beginning after exhaustion
- stop requiring that the pre-rescan pass must have returned a full linear tail over every heap tid

Please review:
- whether the rescan-after-exhaustion pg test now matches the intended graph-first runtime
  semantics
- whether the test still checks enough useful behavior after dropping the retired linear-tail
  assumption
