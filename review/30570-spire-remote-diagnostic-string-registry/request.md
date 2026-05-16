# Review Request: SPIRE remote diagnostic string registry

## Summary

Code checkpoint: `f6ca0f54` (`Centralize SPIRE remote diagnostic strings`)

This slice addresses the review closeout request to reduce drift risk in the SQL-visible remote diagnostic contract strings before libpq execution lands.

- Centralizes target-kind, status, transport, endpoint, candidate-format, descriptor-source, row-locator, and finalization strings in `src/am/ec_spire/root/remote_candidates.rs`.
- Reuses the constants across fanout, target/request/readiness, execution, libpq request, receive, merge-input, and finalization surfaces.
- Leaves placement-state names alone because those still come from `SpirePlacementState` rendering.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search --no-default-features --features pg18`
  - 34 passed; 0 failed; 1399 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only code organization and validation claims.
