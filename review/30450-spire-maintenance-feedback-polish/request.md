# Review Request: SPIRE Maintenance Feedback Polish

## Summary

Task 30 SPIRE Phase 2 now handles two minor reviewer cleanup notes from the
second-half Phase 2 review.

Changes:
- Document the `lock_publish_relation` invariant that callers hold an open
  `Relation` for the guard lifetime and the guard unlocks by captured relid.
- Clarify the maintenance-plan snapshot `planner_message` so `publish_epoch`,
  `next_pid`, and `next_local_vec_seq` are explicitly projected values, not
  committed cursor advances.

## Validation

- `cargo test maintenance_plan_snapshot --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

Addresses minor feedback in
`review/30448-spire-selected-split-input-from-heap-sources/feedback.md`.
No measurement claims.
