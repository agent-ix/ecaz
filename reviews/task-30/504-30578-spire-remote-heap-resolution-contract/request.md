# Review Request: SPIRE remote heap resolution contract

## Summary

Code checkpoint: `c1909663` (`Expose SPIRE remote heap resolution contract`)

This slice makes the final local-vs-remote heap lookup boundary SQL-visible.

- Adds `ec_spire_remote_search_heap_resolution_contract()`.
- Documents that local candidate batches resolve through coordinator-local heap
  lookup.
- Documents that remote candidate batches keep row locators opaque and require
  origin-node heap resolution.
- Extends the existing final-contract PG test to cover both local and remote
  heap resolution rows.
- Updates the Phase 7 task note to mention the heap resolution contract.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote --no-default-features --features pg18`
  - 52 passed; 0 failed; 1386 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only contract and
validation claims.
