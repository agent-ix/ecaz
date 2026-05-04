# Review Request: SPIRE Shared Publish Lock

## Summary

Task 30 SPIRE Phase 2 now has one shared relation publish lock helper for
epoch-publishing paths.

Changes:
- Add `SPIRE_PUBLISH_LOCK_MODE`, `SpireRelationLockGuard`, and
  `lock_publish_relation` in the SPIRE module root.
- Switch insert delta publication to the shared lock helper.
- Switch vacuum delete/cleanup publication to the shared lock helper.

## Validation

- `cargo test maintenance_plan_snapshot --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a small prerequisite
for wiring manual scheduler execution through the same publish-lock contract as
insert and vacuum.
