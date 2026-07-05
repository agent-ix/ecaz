# SPIRE Build Path Complete

## Checkpoint

- Code commit: `5f7fc97f`
  (`Mark SPIRE build path complete`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Task-plan closeout for Phase 1 populated/empty build publication

## Summary

This checkpoint marks the Phase 1 build-path task complete in the Task 30
plan.

The already-landed build path now covers:

- shared centroid training through `src/am/common/training.rs`
- empty build root/control initialization
- populated strict local builds with relation-backed root routing objects,
  V2 leaf objects, placement entries, manifest bundles, and active
  root/control publication
- TurboQuant and RaBitQ assignment payloads through row-local scoring metadata
- explicit populated-build rejection for SPIRE `pq_fastscan` until grouped-PQ
  model metadata is persisted
- active-epoch loading by scans and diagnostics after build publication

## Changed Files

- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`
- `git diff --cached --check` before commit

Tests were not rerun for this documentation-only closeout. The immediately
preceding checkpoint (`30327`) ran the focused PQ-FastScan deferral test and
the full SPIRE PG18 lib suite:

- `cargo test --lib test_ec_spire_pq_fastscan_populated_build_reports_deferral --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1114 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `234 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`

## Notes

- This does not close scan-time PQ-FastScan scorer binding, measured
  recall/latency evidence, insert batching, physical old-epoch cleanup, or
  concurrency stress.
