# 30909: SPIRE Hardening Task Split

## Scope

This packet turns reviewer packet `30896` into durable task planning.

Changes:

- Recasts Phase 11 as the functional CustomScan / ADR-069 distributed
  read-write delivery baseline.
- Adds `task30-phase12-spire-production-hardening.md` for the remaining
  hardening, performance, local-readiness, and runbook work.
- Adds `task30-phase13-spire-aws-verification.md` so AWS/RDS-class
  verification is explicitly last and blocked on Phase 12.
- Updates `plan/tasks/README.md` to list the new SPIRE task phases.

The Phase 12 task file exhaustively folds in the reviewer H1-H12 and P1-P9
items from `30896`, plus the prior local multi-instance, multi-store, operator
runbook, and compatibility cleanup work that was still open in Phase 11.

## Validation

- `git diff --check 55a9284d^ 55a9284d`
  - artifact: `artifacts/git-diff-check.log`

No code or SQL behavior changed in this packet.

## Review Focus

- Confirm AWS/RDS-class verification is now clearly deferred to Phase 13.
- Confirm Phase 12 is exhaustive with respect to reviewer packet `30896`:
  hardening H1-H12, performance P1-P9, local multi-instance readiness,
  multi-store readiness, and operator runbooks.
- Confirm Phase 11 remains understandable as historical/functional delivery
  context rather than the active catch-all for production hardening.
