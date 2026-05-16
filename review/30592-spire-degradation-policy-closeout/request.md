# SPIRE Degradation Policy Closeout

## Summary

This packet marks the Phase 7 graceful degradation policy checklist item
complete now that the implementation has a SQL-visible contract, planner
behavior, summary propagation, and a drift guard.

Changes:

- Marks **Graceful degradation policy** complete in
  `plan/tasks/30-spire-ivf-foundation.md`.
- Records that `ec_spire_remote_degradation_policy_contract()` documents the
  shared strict/degraded placement-state actions.
- Records that
  `remote_degradation_policy_contract_matches_fanout_skip_decisions` guards the
  SQL contract against fanout planner drift.
- Records that mixed local/degraded-skipped coordinator plans preserve
  `degraded_ready` through execution, merge, finalization, heap resolution,
  local heap-candidate, and coordinator result summaries.

## Files

- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `24fb86ec`

- Tests not run for this docs/status-only checkpoint.

Previously cited coverage for the completed behavior:

- `review/30588-spire-degradation-policy-invariant/`:
  - `cargo check --lib --no-default-features --features pg18`
  - `cargo test --lib remote_degradation_policy_contract_matches_fanout_skip_decisions --no-default-features --features pg18`
- `review/30590-spire-coordinator-gate-reuse/` and
  `review/30591-spire-coordinator-execution-reuse/` cover the coordinator gate
  reuse and final-summary paths that now preserve degraded readiness.

## Notes

Coordinator transport and distributed epoch publication remain open Phase 7
items. This packet only closes the policy definition and propagation checklist
item; it does not claim libpq fanout or distributed publish execution is done.
