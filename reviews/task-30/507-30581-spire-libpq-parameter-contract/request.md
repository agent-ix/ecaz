# Review Request: SPIRE libpq parameter contract

## Summary

Code checkpoint: `9a6709c4` (`Expose SPIRE libpq parameter contract`)

This slice makes the remote-search libpq request bind contract SQL-visible.

- Adds `ec_spire_remote_search_libpq_parameter_contract()`.
- Names the six bind parameters used by the request envelope.
- Exposes each parameter's PostgreSQL type, semantic role, and validator.
- Extends the existing receive-contract PG test to validate both parameter and
  result contracts.
- Updates the Phase 7 task note to mention the bind-parameter contract.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote --no-default-features --features pg18`
  - 54 passed; 0 failed; 1386 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only contract and
validation claims.
