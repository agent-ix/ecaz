# Review Request: SPIRE Remote Candidate Receive Validation

- Code commits:
  - `31068c9f` (`Add SPIRE remote candidate receive validation`)
  - `c5b911c1` (`Add SPIRE remote candidate batch merge helper`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint adds the first coordinator receive-boundary validation helper
for compact SPIRE remote-search candidate batches:

- adds `validate_remote_search_candidate_batch`;
- adds `SpireRemoteSearchCandidateBatch`;
- adds `merge_validated_remote_search_candidate_batches`;
- validates positive requested epoch and nonzero/unique selected PIDs;
- requires every received candidate to match the requested epoch;
- requires every candidate node ID to match the fanout target node ID;
- rejects candidate PIDs that were not selected for that target;
- rejects PID 0, object version 0, non-visible assignment flags, empty vec-id,
  empty row locator, and non-finite score;
- validates all target-scoped batches before flattening them into the existing
  global merge helper;
- keeps the existing merge helper scope unchanged;
- adds unit coverage for accepted primary/boundary candidate rows, common
  receive-contract drift cases, validated batch merge, and invalid-batch
  rejection before merge;
- updates the Phase 7 task note to record the receive-boundary contract.

This still does not open libpq connections or perform remote transport. The
helpers are intended to sit between future libpq row decoding and the existing
candidate merge helper. The existing multi-node global `vec_id` uniqueness
precondition remains in force.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/tests.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check that the helper validates the right receive-boundary facts before a
   remote candidate batch can enter the coordinator merge path.
2. Check that the validation remains target-scoped: expected node ID plus the
   fanout-selected PID set.
3. Check that assignment flag validation correctly requires rows that are
   visible/scored by SPIRE scan semantics.
4. Check that the helper does not broaden the merge helper's multi-node
   `vec_id` uniqueness precondition.

## Validation

- `cargo test --lib remote_candidate --no-default-features --features pg18`
  - Result: passed; 7 tests passed, including receive-boundary validation and
    validated batch merge tests.
- `git diff --check`
