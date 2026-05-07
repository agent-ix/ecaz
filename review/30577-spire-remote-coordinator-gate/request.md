# Review Request: SPIRE remote coordinator gate

## Summary

Code checkpoint: `1bdade3e` (`Expose SPIRE remote coordinator gate`)

This slice adds a single SQL-visible coordinator gate that ties the existing
Phase 7 execution, merge, and final heap-fetch readiness surfaces together.

- Adds `ec_spire_remote_search_coordinator_gate_summary(...)`.
- Reports plan and PID counts from the execution summary.
- Carries execution status, merge status, final heap-fetch status, and a
  `next_blocker` value so the coordinator path has one integration gate before
  libpq and remote heap resolution land.
- Covers both local-ready and remote-descriptor-blocked SQL paths.
- Updates the Phase 7 task note to mention the coordinator integration gate.

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
