# Review Request: SPIRE Read Schema Drift Scope Deferral

## Summary

Coder: `coder1`
Topic: `758-c1-spire-read-schema-drift-scope-deferral`
Code commit: `ce0ee95477f2dac287159d26145758e3bfe920b1`
Date: `2026-05-15`

This checkpoint records the Phase 12c.4 scope disposition in the updated
task tracker. It does not add live schema-drift read-path fixtures. Instead,
it marks the 12c.4 coord-only, remote-only, and both-sides READ drift rows as
deferred from Phase 12c with rationale.

The rationale is based on reviewer feedback:

- `31110` found that descriptor registration stores coordinator and remote
  shape fingerprints, but the CustomScan read path does not compare them
  before dispatch.
- `31120` kept 12c.4 as the remaining phase-level scope decision and stated
  the next iteration should focus on the 12c.4 scope decision, reconciliation,
  and closeout.

Because Phase 12c is test-only, adding the READ-path fingerprint guard would
be a production behavior change. The tracker now makes that explicit instead
of leaving the rows looking like forgotten test work.

## Files

- `plan/tasks/task30-phase12c-spire-test-coverage.md`

## Validation

- `git diff --check -- plan/tasks/task30-phase12c-spire-test-coverage.md` passed.
- `rg -n "^- \\[ \\]" plan/tasks/task30-phase12c-spire-test-coverage.md` now returns no unchecked rows.
- No Rust tests were run for this tracker-only scope disposition.

## Review Needs

Please verify that recording 12c.4 as deferred from Phase 12c is acceptable
given feedback `31110`/`31120`, and whether the follow-up production phase
should be named explicitly in a separate tracker.
