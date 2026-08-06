# Task 215 review request: release contract

This packet requests review of the BW64/H8 productionization contract before
the default-change code checkpoint and normal PG18 release A/B.

The candidate is exactly the review-closed Task 206 point: BW64/H8, production
head seed derivation yielding 128 effective seeds, and the already-reviewed
Task 205 `candidate_heap_limit=32` path. The control remains BW4/H100 with
the same L=32 owner-traversal surface. The contract changes no persisted
format or index lifecycle behavior and provides SQL/session rollback to the
current defaults.

Please review `artifacts/release-contract.md` and
`artifacts/manifest.md`. The packet intentionally makes no default-change or
performance claim; those are decided only by packet 003's normal PG18 A/B.
