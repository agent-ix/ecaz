# Task 167 packet 009 — identity collision guard

Coder checkpoint for commit `cd12cd0e3`.

The physical callback now propagates PostgreSQL's `index_unchanged` update hint
through local and remote owner paths. A current physical record is rejected as
`EC_DUPLICATE_VEC_ID` for the normal insert path and can be replaced only when
the callback identifies an unchanged-index update. The remote SQL endpoint
carries the same decision so owner behavior matches coordinator behavior.

Validation is recorded in `artifacts/manifest.md` and `artifacts/validation.log`.

This does not close Task 167. Vector-changing UPDATE detection, cross-owner
backlink publication with abort safety, TC-043 fault/concurrency evidence, and
the required `ecaz bench suite` A/B matrix remain open.
