# Review Request: SPIRE Maintenance Publish Scan Visibility

## Summary

Task 30 SPIRE Phase 2 now verifies that manual maintenance replacement epochs
remain visible to user-facing ordered scans after publish.

Changes:
- Extend the merge publish smoke to force an indexed `ORDER BY embedding <#>`
  scan after the replacement epoch publishes.
- Extend the split publish smoke to force an indexed ordered scan and return 20
  rows from the post-split active epoch.

## Validation

- `cargo pgrx test pg18 maintenance_run`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims.
