# Review Request: SPIRE Local Readiness Evidence Boundaries

Code checkpoint: `8b2f0d52` (`Define SPIRE local readiness evidence labels`)

## Scope

- Adds `docs/SPIRE_LOCAL_READINESS.md`.
- Defines the claim boundaries for:
  - local functionality evidence;
  - local production-readiness smoke evidence;
  - AWS/RDS product-scale evidence.
- States which claims each evidence label may and may not support.
- Pins the Phase 13 entry boundary: AWS/RDS verification should not start by
  implementing missing Phase 12 hardening, and accepted Phase 12 deferrals must
  be repeated in AWS reports.
- Marks the Phase 12.9 documentation row for evidence-label separation
  complete.

## Validation

- `git diff --check 8b2f0d52^ 8b2f0d52`

Packet-local log is under `artifacts/`; see `artifacts/manifest.md` for the
command and result line.

## Review Focus

- Confirm the three evidence labels are strict enough to prevent accidental
  AWS/product-scale claims from local fixtures.
- Confirm the local production-readiness smoke checklist matches the remaining
  Phase 12.9 bundle and counter rows without pretending those rows are already
  complete.
