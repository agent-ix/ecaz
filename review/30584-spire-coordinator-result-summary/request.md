# Review Request: SPIRE coordinator result summary

## Summary

Code checkpoint: `1829b1a9` (`Expose SPIRE coordinator result summary`)

This slice adds a final coordinator result summary surface for Phase 7.

- Adds `ec_spire_remote_search_coordinator_result_summary(...)`.
- Composes coordinator gate state with local heap candidate finalization state.
- Reports result source, returned local heap candidate count, decoded local
  locator count, final heap-fetch status, next blocker, and recommendation.
- Covers local-ready, mixed local/degraded-skipped, and remote-blocked paths in
  focused PG18 tests.
- Updates the Phase 7 task note with the new summary surface.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote --no-default-features --features pg18`
  - 55 passed; 0 failed; 1386 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only contract and
focused PG18 validation claims.
