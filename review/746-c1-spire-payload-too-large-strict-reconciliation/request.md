# Review Request: SPIRE Payload-Too-Large Strict Tracker Reconciliation

agent: coder1
date: 2026-05-15
code_commit: 52a2571c
task: SPIRE task 12c.2.a tracker reconciliation

## Summary

Tracker-only reconciliation for the strict row-byte leg of 12c.2.a
`payload_too_large`.

The updated broken-down tracker still had this row unchecked, but packet `731`
already added the CustomScan large-text cap fixture and reviewer feedback
`31120` accepted that fixture. This packet records that existing evidence on
the 12c.2.a row without closing the separate degraded skip/report row.

## Changes

- Marked the 12c.2.a strict row-byte cap row complete.
- Evidence: `test_ec_spire_customscan_large_text_projection_cap_sql`.
- Accepted packet: `review/731-c1-spire-customscan-large-text-projection`.
- Reviewer feedback: `review/31120-spire-phase12c-batch5-feedback`.

The 12c.2.a degraded-mode row remains unchecked because it still needs
separate evidence for `degraded_skipped_dispatch_count` and the surfaced
matrix-row hint.

## Validation

Passed:

- `git diff --check -- plan/tasks/task30-phase12c-spire-test-coverage.md`

No tests were run because this is tracker-only reconciliation against accepted
existing evidence.

## Review Focus

- Confirm `test_ec_spire_customscan_large_text_projection_cap_sql` is valid
  evidence for the strict 12c.2.a row-byte `remote_payload_too_large` row.
- Confirm leaving the degraded 12c.2.a row unchecked is the right conservative
  tracker state.
