# Review Request: SPIRE Scheduled Execution Leaf Version Validation

## Summary

Task 30 SPIRE Phase 2 now rejects invalid scheduled replacement leaf object
versions during execution-input assembly, before local or relation object writes.

Changes:

- Extend shared scheduled replacement execution validation to reject
  `leaf_object_version == 0`.
- Cover the relation publish-plan input builder rejection for object-version
  drift.
- Update the Task 30 Phase 2 checklist.

## Validation

- `cargo test scheduled_replacement_execution --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet makes no measurement claims. The change is pure execution assembly
validation and does not add PostgreSQL callback coverage.
