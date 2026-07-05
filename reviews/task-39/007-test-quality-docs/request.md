# Review Request: Task 39 Test-Quality Rules

Task: `plan/tasks/39-test-quality-measurement.md`

Implementation commit: `4ca9931f5508e6fe099147b78e991a98c33849d5`

## Scope

This packet turns the reviewer guidance from Task 39 packets 005 and 006 into canonical docs in `docs/hardening.md`:

- manual coverage ratchet sequence and 2 percentage point tolerance semantics,
- mutation triage table shape and required verdicts,
- cross-arch mutation pattern for SIMD/backend decision points,
- flake-hunt seed logging and failure packet requirements.

The cross-arch section records the Task 39 packet 005 SIMD pattern: keep intrinsic execution in host-specific validation lanes, but extract backend eligibility decisions into small pure functions whose boolean mutations are killable on any host.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md`.

- `git --no-pager diff-tree --stat --summary --no-commit-id HEAD`: `docs/hardening.md` changed only.
- `git --no-pager diff HEAD~1 HEAD --check -- docs/hardening.md`: clean.

No Rust tests were run because this is a docs-only policy update.

## Remaining Task 39 Gaps

This packet does not close the larger coverage/mutation surface. Remaining gaps still include AM page codecs, SPIRE storage helpers, storage guard coverage/mutation, the pgrx coverage feasibility decision, pgrx-required AM/cost coverage, and broader mutation cadence/budget calibration.
