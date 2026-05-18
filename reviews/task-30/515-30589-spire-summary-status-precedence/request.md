# SPIRE Summary Status Precedence

## Summary

This packet addresses the 30583/30584 feedback that remote summary status
precedence was open-coded across several summaries.

Changes:

- Adds `SpireRemoteSummaryStatusMode` and
  `SpireRemoteCountRollup::summary_status(...)`.
- Routes the following summaries through the shared helper:
  - request summary
  - readiness summary
  - execution summary
  - libpq request summary
- Adds `remote_summary_status_helper_preserves_precedence` to pin the key
  ordering:
  - `empty_top_k` first
  - descriptor blockers before libpq transport
  - libpq transport before degraded-ready where applicable
  - degraded-ready before ready

SQL-visible status values are unchanged.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/tests.rs`

## Validation

Head SHA: `3934d872`

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_summary_status_helper_preserves_precedence --no-default-features --features pg18`

Result:

- Focused unit test passed: 1 passed; 0 failed; 1444 filtered out.
- PG18 lib check passed.

## Notes

This is a behavior-preserving refactor. It does not address the separate
pipeline redundancy finding; that remains the next larger coordinator slice.
