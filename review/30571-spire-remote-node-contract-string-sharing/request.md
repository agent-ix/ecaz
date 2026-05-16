# Review Request: SPIRE remote node contract string sharing

## Summary

Code checkpoint: `c054fbda` (`Share SPIRE remote node contract strings`)

This slice extends the remote diagnostic string registry into the node capability and publish-readiness surfaces.

- Reuses the shared remote candidate-format string for node capability rows and summaries.
- Reuses shared readiness and descriptor-blocked status strings in node capability summaries, publish-readiness summaries, and empty-node rows.
- Reuses the shared remote descriptor source string for remote node capability rows.

This is a narrow follow-up to the 30564/30568 drift feedback: a future candidate-format or descriptor-status change now touches the shared registry instead of separate literals in the node snapshot projection.

## Files

- `src/am/ec_spire/root/snapshots.rs`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_node --no-default-features --features pg18`
  - 7 passed; 0 failed; 1426 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only code organization and validation claims.
