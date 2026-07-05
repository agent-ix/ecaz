# Task 39 Review Request: Spire Relation Plan Coverage

Code checkpoint: `4b21c27eb5170c1087395034e566a3a8ac399da1`

## Summary

This packet raises the Task 39 coverage gate for the pure Spire local-store
relation planning logic.

Changes:

- Moved the PostgreSQL identifier limit behind a small helper with a pgrx-free
  fallback so the careful harness can test relation-name planning.
- Gated PostgreSQL catalog creation helpers behind PG feature cfgs; production
  PG18 builds still compile that path.
- Reused the production `relation_plan.rs` source and existing relation-plan
  tests in the careful Spire harness.
- Raised `am/ec_spire/storage/relation_plan.rs` from `0.00%` to `82.98%`.

`am/ec_spire/storage/relation_store.rs` remains an open relation-backed PG
coverage gap.

## Evidence

- Focused careful relation-plan tests:
  `artifacts/careful-spire-relation-plan-tests.log`
  - 3 passed, 0 failed.
- Coverage: `artifacts/coverage/summary.txt`
  - `am/ec_spire/storage/relation_plan.rs`: 94 lines, 16 missed, `82.98%`
    line coverage.
- Coverage baseline completeness:
  `artifacts/coverage-baseline-check.log`
  - `coverage baseline complete for 40 critical paths`.
- Production compile check: `artifacts/cargo-check-pg18-bench.log`
  - passed with pre-existing warnings.
- Whitespace check: `artifacts/git-diff-check.log`
  - no whitespace errors.

## Review Notes

Please focus on whether the PG cfg boundary preserves production behavior while
making only the pure planning helpers available to the careful harness.
