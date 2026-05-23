# Review Request: SPIRE CustomScan Raw Pointer Boundaries

## Summary

This checkpoint addresses the soundness-audit feedback for SPIRE CustomScan helpers that accepted PostgreSQL raw pointers through safe private APIs.

Code commit: `ee264b443684cc00923f9f78aa35143dbb84ed5c`

The reviewer was correct: null checks and internal `SAFETY` comments do not make these helpers safe to call. This slice keeps behavior unchanged but restores honest unsafe boundaries around CustomScan planner/executor pointers, plan-private lists, expression nodes, tuple slots, memory contexts, and provider-owned tuple-payload slot writers.

## Scope

- Marked CustomScan planner helper readers unsafe where they inspect `PlannerInfo`, `RelOptInfo`, `RangeTblEntry`, `CustomPath`, `PathTarget`, `Expr`, `List`, and provider plan-private metadata.
- Marked CustomScan executor helper readers/writers unsafe where they inspect `CustomScanState`, `CustomScan`, `ScanState`, `TupleTableSlot`, `MemoryContext`, and DML expression handoff nodes.
- Added explicit call-site safety acknowledgments at PostgreSQL callback boundaries and typed provider handoff points.
- Left the stronger RAII/view redesign for later passes, per current strategy.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed; count increase is expected because this pass makes previously hidden raw-pointer contracts explicit.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
