# Review Request: SPIRE Locked Maintenance Plan Snapshot

## Summary

Task 30 SPIRE Phase 2 now has a no-write locked maintenance preflight surface.

Changes:
- Add `ec_spire_index_locked_maintenance_plan_snapshot(index_oid)`.
- Acquire the shared SPIRE publish lock before loading the active epoch
  snapshot and deriving the checked split/merge maintenance plan.
- Update the Phase 2 checklist.

## Validation

- `cargo test maintenance_plan_snapshot --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This gives the live manual
scheduler entrypoint a matching lock-time preflight surface before replacement
object publication is wired.
