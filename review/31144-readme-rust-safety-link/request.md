# 31144: README Rust Safety Link

## Scope

This packet covers commit `842fdc98`.

Changed file:

- `README.md`

## What Changed

- Added `docs/hardening.md` to the README documentation table as
  "Rust Safety And Quality".
- Linked the final AI/production-readiness statement to the same hardening doc
  so the quality claim points readers at the concrete safety lanes.

## Review Focus

- Confirm the README link text is clear and does not overstate production
  readiness.
- Confirm the final AI statement now points at the safety/hardening docs while
  preserving the warning that the project is not production-ready.

## Validation

- `git diff --check`

No tests were run. This is a README-only documentation change.
