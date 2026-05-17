# Review Request: SPIRE Remote Search Execution Plan

- Code commit: `01c266b0` (`Expose SPIRE remote search execution plan`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint exposes the final pre-libpq executor contract as SQL-visible
diagnostics:

- adds `SpireRemoteSearchExecutionPlanRow`;
- adds `ec_spire_remote_search_execution_plan(...)`;
- derives execution rows from request readiness rows;
- reports local targets as `local_direct`;
- reports remote targets as `libpq_pipeline` with remote index and conninfo
  sources both coming from the future `remote_node_descriptor`;
- reports skipped degraded targets as transport `none`;
- exposes endpoint function and expected candidate format per target;
- preserves descriptor blocking by carrying
  `requires_remote_node_descriptor` through from request readiness;
- adds `SpireRemoteSearchExecutionSummaryRow`;
- adds `ec_spire_remote_search_execution_summary(...)`;
- summarizes local/remote/skipped plan counts, ready/blocked/degraded counts,
  PID counts, query/top-k metadata, consistency mode, and effective status;
- updates the Phase 7 task note with the execution-plan surfaces;
- adds PG18 coverage for blocked remote execution plans and degraded skipped
  execution plans.

This still does not store remote descriptors, expose raw conninfo, open libpq
connections, execute remote SQL, decode remote rows, or resolve final heap rows.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check the execution-plan contract: target kind, transport, endpoint,
   remote-index source, conninfo source, candidate format, and status.
2. Check that remote targets remain blocked by missing descriptors and do not
   imply libpq transport is executable yet.
3. Check that degraded skipped targets use transport `none`, endpoint `none`,
   candidate format `none`, and summarize as `degraded_ready`.
4. Check that no raw connection strings or remote SQL execution behavior are
   exposed.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search_exec --no-default-features --features pg18`
  - Result: passed; 2 tests passed.
- `git diff --check`
