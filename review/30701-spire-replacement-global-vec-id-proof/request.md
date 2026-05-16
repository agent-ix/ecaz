# Review Request: SPIRE Replacement Global Vec-ID Proof

Status: open
Owner: coder1
Head SHA: `8b8b2afaaa4631b5fa10f6662bcb0e6a1ffed6aa`

## Summary

This Phase 11.2 Stage A follow-up closes the remaining scheduled replacement
and replica proof gaps after the `source_identity = 'include'` provider slice.

Key changes:

- Adds test helpers for fixed-width global `SpireVecId` rows.
- Proves replacement row collection preserves global vec IDs from both base
  Leaf V2 rows and live delta rows, while normalizing delta-insert flags before
  replacement materialization.
- Proves merge replacement leaf input preserves global vec IDs in affected leaf
  order.
- Proves split replacement materialization reuses the same global vec ID bytes
  for primary and boundary-replica rows.
- Runs existing node-local namespace tests and marks the Phase 11 Stage A
  verification item closed in the task file.

## Deliberate Limits

- This is proof coverage for existing replacement behavior. It does not change
  the replacement writer code because that path already carries
  `SpireLeafAssignmentRow` values with their assigned vec IDs.
- Distributed remote endpoint, production libpq coordinator execution, and
  remote heap resolution remain later Phase 11 stages.

## Validation

- `cargo fmt`
  - passed; rustfmt still prints existing stable-toolchain warnings for
    unstable import-grouping settings
- `cargo test global_vec_ids --lib`
  - 7 passed, 0 failed
- `cargo test local_vec_ids_by_node --lib`
  - 2 passed, 0 failed
- `git diff --check`
  - passed

## Review Focus

- Is the scheduled replacement proof sufficient to close the Stage A global-ID
  replacement gap?
- Does the split replacement boundary test cover the right primary/replica
  identity invariant?
- Is the Phase 11 task-file closure conservative enough before moving to Stage
  B remote endpoint work?
