# Review Request: SPIRE Coordinator Local Remote Search

- Code commits:
  - `a322b95d` (`Add SPIRE coordinator local remote search path`)
  - `154c5335` (`Add SPIRE coordinator remote-target fail-closed test`)
  - `a0af9d5c` (`Add SPIRE coordinator local search summary`)
  - `a271f798` (`Expose SPIRE degraded coordinator summary status`)
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
- adds test-only placement-node rewrite support and PG18 coverage proving the
  coordinator-local endpoint fails closed before transport when a selected leaf
  is planned for a nonlocal node;
- adds `ec_spire_remote_search_coordinator_local_summary(...)`, returning one
  diagnostic row with local PID count, remote target/PID counts, skipped
  placement count, merge input count, duplicate vec-id count, returned
  candidate count, and status;
- reports `requires_libpq_transport` for nonlocal fanout plans without opening
  local object stores or attempting transport;
- adds PG18 coverage for both local-ready and remote-target summary rows;
- reports `degraded_ready` when degraded-mode fanout skips selected
  unavailable/skipped placements;
- adds test-only consistency-mode rewrite support plus PG18 coverage proving
  degraded skipped placements produce zero candidates while preserving the
  skipped-placement count;
- updates the Phase 7 task note to record the local-only coordinator path and
  the remaining libpq transport boundary.

This does not open libpq connections, execute remote SQL, resolve final heap
rows, or remove the existing global `vec_id` uniqueness precondition for
multi-node candidate merge.

## Files

- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/ec_spire/root/debug.rs`
- `src/am/ec_spire/root/types.rs`
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
4. Check that the test-only remote placement fixture does not weaken production
   local-store validation.
5. Check that the summary status/count contract is useful for the future libpq
   executor and does not imply transport has landed.
6. Check that `degraded_ready` is the right status name for lower-recall
   degraded plans that complete without transport.
7. Check whether the SQL names and row contracts are acceptable as diagnostic
   coordinator bridges before real transport lands.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search --no-default-features --features pg18`
  - Result: passed; 11 tests passed, including the coordinator-local SQL
    endpoint test, the nonlocal remote-target fail-closed test, and the summary
    count/status tests.
- `git diff --check`
