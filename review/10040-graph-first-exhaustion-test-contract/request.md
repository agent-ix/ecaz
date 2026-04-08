# Review Request: Graph-First Exhaustion Test Contract

Commit: `aa8425a`

Scope:
- `src/lib.rs`

Summary:
- update `pg_test_tqhnsw_gettuple_exhaustion_stays_false` to the staged A3 graph-first contract
- keep the exhaustion assertions about repeated `amgettuple` returning `false`
- stop requiring that scan exhaustion implies a full linear tail over every heap tid once the
  graph-owned ordered lane has already started

Please review:
- whether the exhaustion pg test now matches the intended graph-first runtime contract precisely
- whether the test still checks enough real behavior after dropping the retired linear-tail
  assumption
