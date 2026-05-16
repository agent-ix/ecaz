# Review Request: SPIRE remote summary rollups

## Summary

Code checkpoint: `126a99d3` (`Consolidate SPIRE remote summary rollups`)

This slice addresses the 30565/30568 feedback about duplicated remote diagnostic count arithmetic.

- Adds `SpireRemoteCountRollup` in `src/am/ec_spire/root/remote_candidates.rs`.
- Routes request, readiness, execution, and libpq request summaries through shared target/status counting helpers.
- Preserves the existing SQL-visible fields and status precedence while centralizing overflow checks and unknown target/status validation.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search --no-default-features --features pg18`
  - 34 passed; 0 failed; 1399 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only code organization and validation claims.
