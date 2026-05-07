# Review Request: SPIRE remote merge tie-breaker invariant

## Summary

Code checkpoint: `b2e20644` (`Assert SPIRE remote merge tie breaker contract`)

This slice adds the 30568-suggested invariant coverage for the merge-input diagnostic tie-breaker string.

- Extends `test_ec_spire_remote_search_receive_merge_summary` to read `tie_breaker`.
- Asserts the SQL diagnostic reports `score_then_assignment_role_then_epoch_desc_then_node_pid_version_row_locator`, matching the comparator order fixed in 30569.

## Files

- `src/lib.rs`

## Validation

- `cargo test --lib test_ec_spire_remote_search_receive_merge_summary --no-default-features --features pg18`
  - 1 passed; 0 failed; 1432 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only test coverage and validation claims.
