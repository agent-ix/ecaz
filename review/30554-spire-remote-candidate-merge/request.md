# Review Request: SPIRE Remote Candidate Merge

- Code commit: `ae9cec4e` (`Add SPIRE remote candidate merge helper`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 multi-machine placement
- Agent: coder1

## Summary

This checkpoint adds the first production merge helper for compact SPIRE remote
candidate rows:

- adds `root/remote_candidates.rs` and includes it in the SPIRE root module;
- introduces `SpireRemoteSearchMergeResult`;
- adds `merge_remote_search_candidates`, which accepts storage-node candidate
  rows, validates finite scores and nonempty `vec_id` values, dedupes globally
  by `vec_id`, sorts the retained candidates, and applies the final top-k cap;
- ranks lower score first, then primary assignment rows before boundary
  replicas, then newer served epoch/object version, then deterministic
  node/pid/row/locator/vec-id tie-breakers;
- reports input count and duplicate-vec-id count for future coordinator
  diagnostics;
- adds unit coverage for global vec-id dedupe, top-k-after-dedupe behavior,
  primary-vs-boundary tie-breaking, and invalid candidate envelopes;
- updates the Phase 7 task note to record the merge-helper progress while
  keeping coordinator integration and row resolution open.

This does not call the helper from a coordinator yet, does not implement libpq
fanout, and does not resolve remote row locators back to local heap rows.

## Files

- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/tests.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check the global ranking order, especially score ascending, primary before
   boundary replica on ties, newer epoch/object version preferences, and
   deterministic tie-breakers.
2. Check that dedupe by raw `vec_id` bytes is the right contract for the first
   coordinator merge helper.
3. Check that top-k is applied after global dedupe rather than per-node or
   pre-dedupe.
4. Check whether the validation surface should reject more candidate envelope
   fields now, or wait until the libpq caller has a real remote error contract.
5. Check whether `input_count` and `duplicate_vec_id_count` are the right first
   diagnostics for coordinator merge observability.

## Validation

- `cargo test --lib remote_candidate_merge --no-default-features --features pg18`
  - Result: passed; 3 tests passed.
- `cargo check --lib --no-default-features --features pg18`
- `git diff --check`
