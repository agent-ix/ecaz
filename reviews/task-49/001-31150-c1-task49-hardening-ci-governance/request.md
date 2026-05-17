# Review Request: Task 49 hardening CI governance

Code commit: `105ef48f19a1c56091c4aaa9a076de4b4db01d54`

## Summary

This packet implements the first Task 49 governance cleanup.

- Restored `make test` to run full `cargo test`.
- Added `make test-local` as the explicit macOS-safe local subset that keeps
  using `test-hardening-local`.
- Removed the synthetic-only Rudra, Flux, Loom, and Shuttle hardening crates.
- Removed their Makefile/script lanes so they no longer appear as green
  hardening signal.
- Added `make hardening-validate`, which fails if retired synthetic lanes
  return or if retained hardening crates do not import real `src/` code.
- Added `make hardening-tiers-report` and `docs/hardening-governance.md` for
  lane tiering, promotion, demotion, and inventory.
- Marked Task 49 implemented in `plan/tasks/49-hardening-ci-governance.md`.

## Reviewer Focus

- Confirm removing the synthetic harness crates is preferable to preserving
  false signal until Tasks 40/44/45 add real lanes.
- Confirm `make test` now has CI semantics and `make test-local` is the correct
  documented local escape hatch for the macOS pgrx loader issue.
- Confirm `scripts/hardening_validate.sh` is strict enough to catch the same
  synthetic-lane class without blocking retained real-code hardening crates.

## Validation

- `bash scripts/hardening_validate.sh`
  - artifact: `artifacts/hardening-validate.log`
- `make hardening-tiers-report`
  - artifact: `artifacts/hardening-tiers-report.log`
- `make test-local`
  - artifact: `artifacts/test-local.log`
- `make test`
  - artifact: `artifacts/make-test-macos.log`
  - expected local macOS result: fails at pgrx dynamic symbol load after
    invoking full `cargo test`; this confirms the restored target behavior and
    the reason `make test-local` remains documented.
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
