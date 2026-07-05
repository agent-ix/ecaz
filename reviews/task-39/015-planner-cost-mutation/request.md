# Task 39 Review Request: Planner Cost Coverage + Mutation

Code checkpoint: `263c36de197454dbcefa387ba84200b9943f61cf`

## Summary

This packet closes the Task 39 planner-cost-model gap for `src/am/common/cost.rs` in the supported pgrx-free quality lane.

Changes:

- Imported `src/am/common/cost.rs` into `hardening/careful` by cfg-gating pgrx callback glue behind `pg17`/`pg18`.
- Extended the careful cost tests to assert reltuples selection, linear-only cost, graph startup, graph plus tail total, no-tail graph coverage, and compare-type strings.
- Added `src/am/common/cost.rs` to the careful-backed mutation mapping in `scripts/hardening.sh`.
- Raised `am/common/cost.rs` in `fixtures/quality/coverage-baseline.tsv` from 0.00% to 98.98% and documented that live planner callbacks remain outside this pgrx-free coverage surface.

## Evidence

- Coverage: `artifacts/coverage-summary.txt`
  - `am/common/cost.rs`: 98.98% line coverage.
- Focused tests: `artifacts/careful-cost-tests.log`
  - 13 passed, 0 failed.
- Full quality coverage lane: `artifacts/make-coverage.log`
  - `ecaz-cli`: 355 passed.
  - `hardening/careful`: 219 passed.
- Mutation initial run: `artifacts/cost.rs.mutants/mutants.out/missed.txt`
  - 18 missed mutants identified and triaged in `triage.md`.
- Mutation rerun: `artifacts/rerun/cost.rs.mutants/mutants.out/outcomes.json`
  - 58 mutants tested, 0 missed, 52 caught, 6 unviable.
- Baseline checks:
  - `artifacts/coverage-delta-check.log`
  - `artifacts/coverage-baseline-complete.log`
- Production compile check:
  - `artifacts/cargo-check-pg18-bench.log`
- Whitespace check:
  - `artifacts/git-diff-check.log`

## Review Notes

Please focus on whether the cfg-gating preserves the production pg17/pg18 callback surface while allowing the pure planner-cost model to run inside the careful harness.

This packet does not claim live pgrx callback coverage. That remains under the feasibility decision in `reviews/task-39/013-pgrx-coverage-feasibility/`.
