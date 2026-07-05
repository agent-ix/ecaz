# Review Request: AWS Row Segment Snapshot Signature Update

Task: `plan/tasks/85-spire-product-scale-pareto-program.md`
Head SHA: `e07b4be5ee28ae74d85a7b4a601340307f0bb413`

## Summary

This packet updates the retained AWS `1m` database function signature for
`ec_spire_index_scan_leaf_candidate_snapshot(oid, real[])` without dropping the
retained corpus or index.

The packet applies the append-only SQL signature for the two row-segment
counters added in packet 017:

- `leaf_row_segment_read_count`
- `leaf_row_segment_read_bytes`

## Evidence

- `artifacts/cloud-install-append-only-signature.log`: installed the current
  Task 85 branch on the retained DB host with `--skip-extension-recreate`.
- `artifacts/aws-row-segment-signature-update/apply-leaf-candidate-snapshot-signature.log`:
  transactional SQL signature replacement succeeded.
- `artifacts/aws-row-segment-signature-update/verify-leaf-candidate-snapshot-segments.log`:
  SQL can now project the appended row-segment counter columns.
- `artifacts/cloud-status-after-resume.log`: AWS profile was running for the
  update window.

## Notes

This packet proves the retained DB can expose the appended columns. Packet 019
contains the full q500 measurement that uses the corrected signature.
