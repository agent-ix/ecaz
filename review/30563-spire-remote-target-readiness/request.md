# Review Request: SPIRE Remote Target and Request Readiness

- Code commits:
  - `e3999442` (`Expose SPIRE remote target readiness`)
  - `7a80db8a` (`Expose SPIRE remote request readiness`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint joins remote-search target/request fanout with remote-node
readiness diagnostics before libpq execution lands:

- adds `SpireRemoteSearchTargetReadinessRow`;
- adds `SpireRemoteSearchRequestReadinessRow`;
- adds `remote_search_target_readiness_rows`;
- adds `remote_search_request_readiness_rows`;
- exports SQL function
  `ec_spire_remote_search_target_readiness(index_oid, requested_epoch,
  selected_pids, consistency_mode)`;
- exports SQL function
  `ec_spire_remote_search_request_readiness(index_oid, requested_epoch, query,
  selected_pids, top_k, consistency_mode)`;
- reuses `ec_spire_remote_search_target_plan(...)` target grouping;
- joins each target row to `ec_spire_remote_node_snapshot(...)` by `node_id`;
- reports target shape plus node kind, descriptor state, node status, and
  effective target readiness status;
- binds target readiness rows to request metadata: query dimension, top-k
  budget, consistency mode, endpoint function, and effective request readiness
  status;
- leaves local targets as `ready`;
- reports nonzero remote targets as `requires_remote_node_descriptor` while the
  durable descriptor catalog is absent;
- preserves degraded skipped targets as `degraded_skipped` instead of turning
  placement skips into node-descriptor failures;
- updates the Phase 7 task note with target/request-readiness diagnostics;
- adds PG18 coverage for mixed local/remote target and request readiness, plus
  degraded skipped target and request readiness.

This remains a diagnostic/planning surface. It does not add durable remote-node
descriptors, raw conninfo storage, health checks, libpq connections, or remote
SQL execution.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check that target-readiness rows preserve the target-plan grouping and
   selected PID arrays.
2. Check that request-readiness rows correctly add query/top-k/endpoint
   metadata without serializing query vectors into logs.
3. Check the status precedence: degraded skipped placements stay
   `degraded_skipped`; executable targets inherit node readiness before
   transport readiness.
4. Check that missing remote descriptors are explicit through
   `requires_remote_node_descriptor` and do not imply libpq execution exists.
5. Check that the join to `ec_spire_remote_node_snapshot(...)` is appropriate
   for this pre-descriptor checkpoint.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search --no-default-features --features pg18`
  - Result: passed; 21 tests passed, including target/request readiness remote
    missing-descriptor and degraded-skipped PG tests.
- `git diff --check`
