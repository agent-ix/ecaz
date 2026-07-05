# Review Request: SPIRE Read Schema Drift Phase 13 Gate

## Summary

Coder: `coder1`
Topic: `759-c1-spire-read-schema-drift-phase13-gate`
Code commit: `7df89d5ad2f1181af6253232d7b8d6199529994a`
Date: `2026-05-15`

This checkpoint adds the explicit Phase 13 handoff row for the Phase 12c.4
READ schema-drift scope decision. Reviewer feedback `31110` said that, if
12c.4 is deferred out of the Phase 12c test-only scope, the deferral should be
tracked in the Phase 13 entry criteria. Packet `758` recorded the Phase 12c
side of that disposition; this packet records the Phase 13 gate.

The new entry gate requires one of two outcomes before AWS verification claims
can proceed:

- the CustomScan read-path fingerprint guard lands with coord-only,
  remote-only, and both-sides drift fixtures; or
- the reviewer-accepted Phase 12c deferral is repeated in the AWS report with
  operator impact.

## Files

- `plan/tasks/task30-phase13-spire-aws-verification.md`

## Validation

- `git diff --check -- plan/tasks/task30-phase13-spire-aws-verification.md`
  passed.
- `rg -n "12c\\.4|READ schema-drift|fingerprint guard|AWS report" plan/tasks/task30-phase13-spire-aws-verification.md`
  shows the new explicit entry-gate row and the existing AWS-report deferral
  exit criterion.

No Rust tests were run because this is a tracker-only handoff.

## Review Needs

Please verify that the Phase 13 entry-gate row satisfies the 12c.4 tracking
request from feedback `31110`, without overclaiming live READ schema-drift
coverage in Phase 12c.
