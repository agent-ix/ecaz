# Task 63 HNSW RaBitQ Review Feedback Response

## Summary

This packet responds to reviewer feedback
`reviews/task-63/022-hnsw-rabitq-suite-audit-handoff/feedback/2026-05-27-01-reviewer.md`.

Code review outcome:

- HNSW RaBitQ implementation was approved.
- Task closeout remains blocked on publishable faster-host benchmark evidence
  and a final recommend/experimental/shelve decision row.
- The reviewer confirmed build, scan, live insert, and vacuum are implemented.

Documentation update:

- `docs/usage.md` now states that HNSW RaBitQ derives search codes from raw
  source vectors, so `tqvector` inputs need `build_source_column` for bulk
  build and raw source data for live inserts, or callers should index
  `ecvector` directly.
- `README.md` now carries the same source-vector caveat near the access-method
  storage-format summary.

No benchmarks were run. The untracked AMD-local benchmark artifacts remain
baseline/tuning output only and are not cited as Task 63 acceptance evidence.

## Validation

Not run. This is a documentation-only feedback response.
