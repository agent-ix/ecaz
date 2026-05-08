# Review Request: SPIRE degraded heap status

## Summary

Code checkpoint: `c181b855` (`Preserve SPIRE degraded heap status`)

This slice fixes a status propagation gap in the Phase 7 coordinator summaries.

- Preserves `degraded_ready` when a plan has local executable work plus
  degraded-skipped placements.
- Carries the degraded status through merge input, finalization, heap
  resolution, and local heap candidate summaries.
- Keeps final local heap fetch status as `local_ready` for the executable local
  work.
- Adds focused PG18 coverage for a mixed local/degraded-skipped plan.
- Updates the Phase 7 task note with the status-preservation behavior.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote --no-default-features --features pg18`
  - 55 passed; 0 failed; 1386 filtered out
- `git diff --check`

## Notes

No measurement artifacts are included; this packet makes only behavior and
focused PG18 validation claims.
