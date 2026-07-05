# Review Request: SPIRE Bounded Tie-Break Ordering

Code checkpoint: `c06057ac` (`Lock SPIRE bounded tie-break ordering`)

## Scope

- Completes the Phase 10.1 deterministic ordering and boundary-replica
  tie-break item.
- Adds a bounded-dedupe regression where a boundary replica arrives before the
  primary row with the same vec-id and score.
- Verifies the bounded accumulator keeps the primary row under the candidate
  cap, preserving the existing `scored_candidate_cmp` role tie-break.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 rank_routed_leaf_rows_by_ip_keeps_primary_tie_break_under_bounded_dedupe --lib`
- `cargo test --no-default-features --features pg18 tie_break --lib`

## Review Focus

- Confirm the regression locks the relevant Phase 10.1 behavior without
  over-specifying unrelated heap/TID ordering.
- Confirm marking the Phase 10.1 tie-break item complete is reasonable with
  the existing comparator test plus this bounded-dedupe test.
