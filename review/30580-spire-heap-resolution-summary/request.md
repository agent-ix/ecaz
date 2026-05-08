# Review Request: SPIRE heap resolution summary

## Summary

Code checkpoint: `cc4df037` (`Summarize SPIRE heap resolution readiness`)

This slice adds a coordinator-facing heap resolution summary over the local
decoded-locator path and the remote-origin blocker.

- Adds `ec_spire_remote_search_heap_resolution_summary(...)`.
- Aggregates local/remote/skipped plan counts and local/remote PID counts.
- Reports decoded local locator count when the plan is local-only and ready.
- Reports remote heap resolution status as the current blocker when remote work
  is still behind descriptors or libpq transport.
- Covers local-ready and mixed remote-blocked SQL paths.

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
