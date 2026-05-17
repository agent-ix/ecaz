# Review Request: Task 41 scan diagnostic relation guards

Code commit: `cc16a6ad90eab3c79f3514713aca147aba9cb7eb`

## Summary

This packet continues Task 41 by migrating the next SPIRE scan diagnostic
cluster in `src/lib.rs` from raw `Relation` open/close pairs to
`AccessShareIndexRelation`.

- Migrated scan placement, selected PID placement, local-store execution, and
  read-overlap diagnostics.
- Migrated scan routing and scan pipeline diagnostics.
- Kept the relation guard live while AM diagnostic helpers materialize rows,
  then dropped it before tuple conversion.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4614 entries.
- After: 4602 entries.
- Net change: 12 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm `AccessShareIndexRelation::as_ptr()` is used only while the guard is
  in scope for AM diagnostic helper calls.
- Confirm row materialization still happens before the guard drops and tuple
  conversion does not depend on the relation pointer.
- Confirm the existing missing-OID early return in pipeline diagnostics is
  preserved.

## Validation

- `bash scripts/unsafe_baseline_report.sh artifacts/unsafe-baseline-before.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
