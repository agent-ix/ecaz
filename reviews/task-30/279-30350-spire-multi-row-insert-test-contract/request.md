# SPIRE Multi-Row Insert Test Contract

## Checkpoint

- Code commit: `60bb3645`
  (`Clarify SPIRE multi-row insert test contract`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Round review follow-up for packet `30319`

## Summary

This checkpoint adds an explicit comment to
`test_ec_spire_insert_after_build_multi_row_epoch_progression`.

The test asserts the current no-batching contract: PostgreSQL invokes
`aminsert` once per row, so one multi-row SQL `INSERT` currently publishes one
delta epoch per row. The comment now says insert batching should update that
expectation deliberately when batching lands.

## Changed Files

- `src/lib.rs`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `git diff --check`

Tests were not run because this is a comment-only test-maintenance change.

## Notes

- This responds to the round-review note that the test should explain why a
  future batching implementation is expected to change its epoch assertion.
