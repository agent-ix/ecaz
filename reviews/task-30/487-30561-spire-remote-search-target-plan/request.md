# Review Request: SPIRE Remote Search Target, Request, and Summary Plan

- Code commits:
  - `c2f4b9e5` (`Expose SPIRE remote search target plan`)
  - `57b3de39` (`Expose SPIRE remote search request plan`)
  - `41050f62` (`Expose SPIRE remote search request summary`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint exposes the coordinator fanout plan at target/request
granularity and request-level summary readiness:

- adds `SpireRemoteSearchTargetPlanRow`;
- adds `SpireRemoteSearchRequestPlanRow`;
- adds `SpireRemoteSearchRequestSummaryRow`;
- adds `remote_search_target_plan_rows`;
- adds `remote_search_request_plan_rows`;
- adds `remote_search_request_summary_row`;
- exports SQL function
  `ec_spire_remote_search_target_plan(index_oid, requested_epoch,
  selected_pids, consistency_mode)`;
- exports SQL function
  `ec_spire_remote_search_request_plan(index_oid, requested_epoch, query,
  selected_pids, top_k, consistency_mode)`;
- exports SQL function
  `ec_spire_remote_search_request_summary(index_oid, requested_epoch, query,
  selected_pids, top_k, consistency_mode)`;
- emits one `local` row for all local selected PIDs;
- emits one `remote` row per remote node target with its selected PID array;
- emits `skipped` rows grouped by `(node_id, placement_state)` for degraded
  unavailable/skipped placements;
- reports `pid_count`, `placement_state`, and target `status`;
- uses `requires_libpq_transport` for remote targets and `degraded_skipped` for
  skipped degraded groups;
- binds target rows to the storage-node endpoint contract by reporting query
  dimension, top-k budget, consistency mode, endpoint function, and status;
- reports endpoint function `ec_spire_remote_search` for executable local/remote
  targets and `none` for degraded skipped groups;
- aggregates request readiness into one coordinator gating row with request
  counts, local/remote/skipped PID counts, executable PID count, query
  dimension, top-k budget, consistency mode, and status;
- reports summary statuses `ready`, `requires_libpq_transport`,
  `degraded_ready`, and `empty_top_k`;
- adds PG18 coverage for mixed local/remote target grouping and degraded
  skipped grouping;
- adds PG18 coverage for request-plan endpoint/top-k/query-dimension reporting
  and degraded skipped request rows;
- adds PG18 coverage for request-summary mixed local/remote readiness and
  degraded skipped readiness;
- updates the Phase 7 task note to record target-level, request-level, and
  request-summary SQL diagnostics.

This still does not open libpq connections or execute remote SQL. These are the
target and request-shape contracts the future libpq pipeline executor should
consume.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check the row contract: target kind, node ID, selected PID array, PID count,
   placement state, and status.
2. Check that remote target rows are grouped by node ID and preserve selected
   PID order within each target.
3. Check that degraded skipped rows group by node/state and remain distinct
   from executable local/remote targets.
4. Check that request-plan rows correctly bind target groups to
   `ec_spire_remote_search` inputs without serializing query vectors into logs.
5. Check that request-summary counts and status match the detail request-plan
   rows, especially mixed local/remote and degraded skipped cases.
6. Check that the surfaces remain diagnostic and do not imply libpq transport
   has landed.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search --no-default-features --features pg18`
  - Result: passed; 17 tests passed, including target-plan local/remote,
    target-plan degraded-skipped, request-plan contract, and request-plan
    degraded-skipped tests, plus request-summary mixed local/remote and
    degraded-skipped tests.
- `git diff --check`
