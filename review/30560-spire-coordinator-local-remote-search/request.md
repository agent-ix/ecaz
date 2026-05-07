# Review Request: SPIRE Coordinator Local Remote Search

- Code commit: `a322b95d` (`Add SPIRE coordinator local remote search path`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint adds the first executable coordinator-side remote-search path,
limited intentionally to local-only fanout:

- refactors the storage-node `ec_spire_remote_search` implementation to share
  a result-returning helper;
- adds `remote_search_coordinator_local_candidates`;
- exports SQL function
  `ec_spire_remote_search_coordinator_local(index_oid, requested_epoch, query,
  selected_pids, top_k, consistency_mode)`;
- plans selected leaves through the coordinator fanout planner;
- fails closed if the plan contains any nonlocal remote target, because libpq
  transport has not landed yet;
- executes the local target batch through the existing selected-leaf candidate
  collector;
- validates the local batch through the receive-boundary helper;
- applies the validated coordinator batch merge helper before returning rows;
- adds PG18 SQL coverage that compares the coordinator-local endpoint with the
  storage-node endpoint on the same local-only fanout request;
- updates the Phase 7 task note to record the local-only coordinator path and
  the remaining libpq transport boundary.

This does not open libpq connections, execute remote SQL, resolve final heap
rows, or remove the existing global `vec_id` uniqueness precondition for
multi-node candidate merge.

## Files

- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check that the new SQL endpoint is clearly local-only and fails before
   remote-target execution.
2. Check that coordinator-local execution uses the same fanout, receive
   validation, and merge boundaries that the future libpq path should use.
3. Check that the storage-node endpoint behavior remains unchanged after the
   result-helper refactor.
4. Check whether the SQL name and row contract are acceptable as a diagnostic
   coordinator bridge before real transport lands.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search --no-default-features --features pg18`
  - Result: passed; 8 tests passed, including the new coordinator-local SQL
    endpoint test.
- `git diff --check`
