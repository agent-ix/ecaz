# Review Request: SPIRE 12c Sign Convention Tracker Reconciliation

- agent: coder1
- date: 2026-05-14
- code commit: `520b04b870ca7d1802f85ae6a09ef0e4f65af755`
- task rows: closes `12c.6.c`

## Summary

Tracker-only reconciliation for `12c.6.c`.

The requested sign-convention extensions already exist in
`src/am/ec_spire/coordinator/tests.rs` inside
`remote_heap_exact_score_uses_orderby_negative_inner_product`.

## Evidence

- `src/am/ec_spire/coordinator/tests.rs:120`
  - Adds a 128-dimensional query/source vector and pins the expected score to
    `-707_264.0`.
- `src/am/ec_spire/coordinator/tests.rs:126`
  - Asserts source-side NaN rejection with the explicit non-finite source-vector
    error.
- `src/am/ec_spire/coordinator/tests.rs:132`
  - Asserts query-side NaN rejection through a non-finite score error.
- `src/am/ec_spire/coordinator/tests.rs:138`
  - Asserts dimension mismatch rejection with a clear query-vs-heap dimension
    message.

## Changes

- Checked the three `12c.6.c` bullets in
  `plan/tasks/task30-phase12c-spire-test-coverage.md`.
- No test code changed in this checkpoint.

## Validation

- `git diff --check -- plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- No compile or runtime test was run for this tracker-only checkpoint; the
  request points to existing unit-test evidence only.

## Review Focus

- Confirm the existing unit-test assertions close `12c.6.c`.
- Confirm query-side NaN rejection via non-finite score is acceptable for the
  tracker’s “AM must refuse” requirement.
