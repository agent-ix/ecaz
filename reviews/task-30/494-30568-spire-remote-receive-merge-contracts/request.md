# Review Request: SPIRE Remote Receive/Merge Contracts

- Code commit: `29a47856` (`Expose SPIRE remote receive merge contracts`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint batches three adjacent receive/merge contract surfaces after the
libpq request-envelope diagnostic:

- adds `SpireRemoteSearchLibpqResultContractRow`;
- adds `ec_spire_remote_search_libpq_result_contract()`;
- exposes the expected 9-column remote result schema, semantic role, nullability,
  and validator for each `ec_spire_remote_search` result column;
- adds `SpireRemoteSearchReceivePlanRow`;
- adds `ec_spire_remote_search_receive_plan(...)`;
- derives per-remote receive plans from the libpq request plan;
- reports selected PIDs, expected candidate format, expected result-column
  count, batch validator function, opaque row-locator policy, and status;
- adds `SpireRemoteSearchMergeInputSummaryRow`;
- adds `ec_spire_remote_search_merge_input_summary(...)`;
- summarizes local/remote/skipped batches, ready/blocked counts, PID counts,
  merge helper, dedupe key, tie-breaker, top-k, and effective status;
- updates the Phase 7 task note with the receive/merge surfaces;
- adds PG18 coverage for the result contract, descriptor-blocked receive plans,
  and merge-input summary.

This still does not call libpq, execute remote SQL, decode remote rows, resolve
remote row locators, or fetch final heap rows.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check the result contract column order, types, semantic roles, and validators
   against `ec_spire_remote_search(...)`.
2. Check that receive plans remain remote-only and blocked until descriptors
   exist.
3. Check that row locators remain explicitly opaque origin-node bytes.
4. Check the merge-input summary contract: merge helper, dedupe key, tie-breaker,
   batch counts, and status propagation.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search_receive --no-default-features --features pg18`
  - Result: passed; 3 tests passed.
- `git diff --check`
