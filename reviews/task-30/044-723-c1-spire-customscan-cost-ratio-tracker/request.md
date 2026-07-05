# Review Request: SPIRE 12c CustomScan Cost Ratio Tracker

- agent: coder1
- date: 2026-05-14
- code commit: `6da83305e6d2360ac337fc70f4d5ba91b4ee0803`
- task rows: closes `12c.10.b`

## Summary

Tracker-only reconciliation for `12c.10.b`.

The requested ratio-based CustomScan cost tests already exist in
`src/am/ec_spire/custom_scan/tests.rs`, replacing the older loose `>` style
checks described by the tracker.

## Evidence

- `src/am/ec_spire/custom_scan/tests.rs:372`
  - `custom_scan_cost_scales_proportionally_with_remote_fanout`
  - Computes startup and total cost ratios for 1-node/4-placement versus
    4-node/16-placement fanout and asserts expected ratio bands.
- `src/am/ec_spire/custom_scan/tests.rs:422`
  - `custom_scan_cost_scales_with_output_rows_without_moving_startup`
  - Subtracts startup cost and asserts the variable-cost ratio tracks the
    output-row ratio.
- `src/am/ec_spire/custom_scan/tests.rs:452`
  - `custom_scan_cost_accounts_proportionally_for_projected_tuple_width`
  - Computes expected tuple-width delta and asserts actual cost delta matches
    proportionally.

## Changes

- Checked the three `12c.10.b` bullets in
  `plan/tasks/task30-phase12c-spire-test-coverage.md`.
- No test code changed in this checkpoint.

## Validation

- `git diff --check -- plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- No compile or runtime test was run for this tracker-only checkpoint; the
  request points to existing unit-test evidence only.

## Review Focus

- Confirm the existing CustomScan unit tests satisfy the `12c.10.b` ratio
  requirements.
- Confirm no additional tracker text is needed for this row.
