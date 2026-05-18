# Review Request: Task 43 Miri Depth Lanes

## Summary

This checkpoint adds the Task 43 depth-lane shell and Makefile infrastructure:

- `make miri-tree` runs the existing `miri_` prefix with
  `-Zmiri-tree-borrows`.
- `make miri-many-seeds` runs the existing `miri_` prefix with
  `-Zmiri-many-seeds=${MIRI_MANY_SEEDS:-128}`.
- `make miri-full` runs default Miri, Tree Borrows, and many-seeds.
- `hardening-nightly-local` now uses `miri-full`.
- `scripts/hardening.sh` discovers `cargo` and `rustup` from `PATH`, with the
  old Homebrew paths preserved as fallback.
- `docs/hardening.md`, `docs/hardening-governance.md`, and
  `scripts/hardening_tiers_report.sh` document the new lanes and triage
  expectations.

## Review Focus

- Confirm `miri-full` is the right aggregate for `hardening-nightly-local`.
- Confirm the `MIRIFLAGS` composition is acceptable when callers provide
  existing Miri flags.
- Confirm the PATH-based rustup discovery is acceptable on Linux while keeping
  the macOS fallback.

## Validation

Validation artifacts are in `artifacts/` and summarized by
`artifacts/manifest.md`.

- `bash -n scripts/hardening.sh` passed.
- `make -n miri-tree miri-many-seeds miri-full hardening-nightly-local` passed.
- `make -n hardening-nightly-local` shows `bash scripts/hardening.sh miri-full`.
- `bash scripts/hardening_validate.sh` passed.
- `bash scripts/hardening_tiers_report.sh` lists `miri-tree` and
  `miri-many-seeds`.

No full Miri execution is claimed in this packet. This is an infrastructure
checkpoint; later coverage packets should store default/Tree/many-seeds Miri
logs separately when they add or promote tests.
