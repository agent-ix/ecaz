# Review Request: SPIRE local heap resolution plan

## Summary

Code checkpoint: `c7415182` (`Expose SPIRE local heap resolution plan`)

This slice makes the executable coordinator-local heap resolution bridge
SQL-visible while keeping remote-origin heap resolution behind the existing
contract.

- Adds `ec_spire_remote_search_local_heap_resolution_plan(...)`.
- Reuses the coordinator-local candidate path and merge helper.
- Decodes local six-byte row locators into heap block/offset work items.
- Reports `coordinator_local_heap` ownership and `ready` status for local
  resolved candidates.
- Updates the Phase 7 task note to distinguish local decoded locators from
  remote-origin resolution.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/hierarchy_snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote --no-default-features --features pg18`
  - 53 passed; 0 failed; 1386 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only contract and
validation claims.
