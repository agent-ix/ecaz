# Review Request: SPIRE Phase 12c Closeout Audit

## Summary

Coder: `coder1`
Topic: `762-c1-spire-phase12c-closeout-audit`
Code commit: `aac40104fea270765672e163ef3bddaaa0ab559b`
Date: `2026-05-15`

This packet is the coder-side closeout audit for SPIRE task 12c test coverage.
It does not close the review request; it asks the reviewer to accept the final
state or identify the next required correction.

## Objective Restated

Complete SPIRE Phase 12c by adding or reconciling test coverage for the atomic
rows in `plan/tasks/task30-phase12c-spire-test-coverage.md`, preserving the
test-only scope, keeping SPIRE-side test files under the file-size target where
this phase touched them, and publishing review packets for the resulting
changes.

## Prompt-to-Artifact Checklist

- Updated broken-down task file: `plan/tasks/task30-phase12c-spire-test-coverage.md`
  is the 896-line atomic tracker introduced by `d0a4f7fa`; current status is
  coder-complete pending reviewer acceptance of the 12c.4 deferral and this
  closeout audit.
- Atomic row closure: `rg -n "^- \\[ \\]" plan/tasks/task30-phase12c-spire-test-coverage.md`
  returns no unchecked rows.
- Test-only scope: Phase 12c production changes are limited to test fixtures,
  test modules, and `#[cfg(test)]` / testability hooks. The only remaining
  non-live coverage item is 12c.4 READ schema drift, explicitly marked as a
  proposed production-behavior deferral rather than a test fixture.
- 12c.4 READ schema drift: packet `758` records the Phase 12c deferral
  rationale from reviewer feedback `31110`/`31120`; packet `759` adds the
  explicit Phase 13 entry-gate row requiring either live READ-path fingerprint
  fixtures or a reviewer-accepted deferral repeated in the AWS report with
  operator impact.
- Review packets: packets `705` through `761` cover the fixture and
  reconciliation slices; packets `758`/`759` cover the 12c.4 deferral/handoff;
  packets `760`/`761` cover file-size cleanup.
- File-size discipline: `src/tests/mod.rs` is now 2492 lines,
  `src/tests/remote_search/contracts.rs` is 1657 lines, and
  `src/tests/remote_search/contracts_libpq.rs` is 1208 lines. The remaining
  >2500-line files in the audit are HNSW test files outside the Phase 12c
  SPIRE scope.
- Validation: latest mechanical splits passed `cargo fmt --check` and focused
  `cargo test --features "pg18 pg_test" --no-default-features ... --no-run`
  compiles. Earlier PG18 runtime attempts for affected CustomScan fixtures
  still hit the environment loader error `undefined symbol: pg_re_throw`, so
  the closeout relies on accepted prior runtime packets plus no-run compiles
  for the final mechanical splits.
- Push visibility: `git ls-remote origin refs/heads/task-30-spire` reports
  `aac40104fea270765672e163ef3bddaaa0ab559b`, matching local `HEAD`.

## Review Needs

Please verify:

- the Phase 12c atomic tracker can be accepted with 12c.4 recorded as a
  deferral instead of live READ schema-drift coverage;
- the Phase 13 gate from packet `759` is sufficient tracking for that deferral;
- the final file-size cleanup satisfies the user's test-bloat concern for the
  SPIRE-side files touched by this phase;
- no additional Phase 12c fixture or reconciliation row remains before Phase
  12c can be considered closed by reviewer.
