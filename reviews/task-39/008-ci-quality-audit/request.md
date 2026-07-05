# Review Request: Task 39 CI Quality Audit

Task: `plan/tasks/39-test-quality-measurement.md`

Implementation commit: `d934addc77a7d7e64e8d9013f9e77c0be2e318a7`

## Scope

This slice hardens the Task 39 CI/nightly contract:

- adds `scripts/check_task39_quality_ci.py` to audit that CI preserves:
  - per-PR `make coverage`, baseline completeness, and coverage delta gate;
  - weekly/manual `make mutants-full`;
  - nightly/manual `make flake-hunt` with the expected 8-seed/10-second budget;
  - artifact uploads for coverage, mutation, and flake-hunt lanes.
- wires the audit as `make test-quality-ci-audit`.
- makes `make flake-hunt` write `manifest.txt` and `expanded-commands.txt` under `target/quality/flake-hunt`.
- uploads `target/quality/flake-hunt` from the CI flake-hunt job.
- documents the flake-hunt artifact contract in `docs/hardening.md`.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md`.

- `make test-quality-ci-audit`: passed.
- `make -n coverage coverage-baseline-check test-quality-ci-audit mutants-full flake-hunt`: confirms all quality entrypoints expand.
- `bash -n scripts/hardening.sh`: passed.
- `python3 -m py_compile scripts/check_task39_quality_ci.py`: passed.
- `git diff --check HEAD~1 HEAD`: clean.

The full local flake-hunt sweep was not run: this environment is missing `cargo-fuzz`, and the default `cargo` is not the rustup shim needed for `cargo +nightly`. CI installs those tools before running the nightly/manual lane.

## Remaining Task 39 Gaps

This packet does not close every Task 39 coverage/mutation target. Remaining work includes PG18/pgrx coverage feasibility for callback-heavy paths, AM page codec and SPIRE storage coverage raises, storage guard coverage/mutation, broader mutation triage beyond the SIMD packet, and burn-in evidence from scheduled CI runs.
