---
topic: spire-phase12-operator-compat-cleanup
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
stage: phase-12.1
status: open
---

# Review Request: SPIRE Phase 12 Operator Compatibility Cleanup

## Scope

Docs-only Phase 12.1 checkpoint for commit `ef2b9268`
(`Document SPIRE Phase 12 operator compatibility`).

This slice processes the Phase 12.1 operator-compatibility cleanup and the
small planning observations from reviewer packet `30909` that could be resolved
without code behavior changes:

- expands the 0.1.1 -> 0.1.2 upgrade-script comment to explain why
  `ec_spire_remote_row_materialization` was created for the Shape-A AM mirror
  path and why the Shape-B CustomScan path drops it;
- documents the diagnostic rename from
  `requires_remote_row_materialization` / `remote_row_materialization` to
  `requires_custom_scan_tuple_delivery` / `custom_scan_tuple_delivery`;
- documents the removed row-materialization and mirror-sync operator-entrypoint
  rows so operator consumers do not expect the old contract-row count;
- records the 0.1.x compatibility window for zero-valued
  `row_materialization_*` cleanup columns and schedules their future 0.2.x
  removal;
- cross-links packet `30895` CustomScan matrix evidence to definition packets
  `30770`, `30772`, and `30773`;
- updates the Phase 12 tracker to mark the Phase 12.1 docs cleanup complete;
- adds measurable exit criteria for the reviewer-flagged "evaluate"/"decide"
  items: JSON fallback retirement, placement-table partitioning evaluation,
  and EvalPlanQual/recheck decision evidence.

No runtime code, SQL objects, or tests changed.

## Files

- `ecaz--0.1.1--0.1.2.sql`
- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`

No tests were run; this is a docs/comment/tracker-only checkpoint.

## Review Focus

- Confirm the operator-facing compatibility notes are accurate for the
  CustomScan pivot and do not imply the removed AM mirror path still exists.
- Confirm marking Phase 12.1 complete is justified by the cited packet and docs
  evidence.
- Confirm the added exit criteria are concrete enough to satisfy reviewer
  packet `30909` observations O3, O4, and O7.
