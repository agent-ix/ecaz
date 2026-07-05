# Review Request: SPIRE local heap candidates

## Summary

Code checkpoint: `65210755` (`Expose SPIRE local heap candidates`)

This slice advances the Phase 7 local coordinator finalization boundary.

- Adds `ec_spire_remote_search_local_heap_candidates(...)`.
- Adds `ec_spire_remote_search_local_heap_candidate_summary(...)`.
- Decodes coordinator-local opaque row locators into heap block/offset work
  items while preserving ranked candidate metadata.
- Keeps remote-target plans fail-closed: the summary reports the existing
  remote descriptor/libpq blocker and returns zero local heap candidates.
- Treats degraded-local plans with skipped placements as eligible for local
  heap-row work once the skipped placement accounting is already visible.
- Updates the Phase 7 task note with the new local heap candidate surface.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
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
focused PG18 validation claims.
