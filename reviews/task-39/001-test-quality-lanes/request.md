# Review Request: Task 39 Test Quality Lanes

## Scope

This packet requests review for commit
`80d0fe0c002edd3ba3466d8fe2694b5dbcb59410`, which adds the first Task 39
test-quality measurement entrypoints:

- `make coverage` and `make coverage-report` via `cargo-llvm-cov`;
- `make mutants` and `make mutants-full` via `cargo-mutants`;
- `make flake-hunt` for seeded proptest plus short fuzz reruns;
- `docs/hardening.md` documentation for gate level, artifacts, and current
  interpretation;
- `scripts/install_hardening_tools.sh --check` visibility for
  `cargo-llvm-cov` and `cargo-mutants`.

## Review Focus

- Whether the initial coverage scope is accurately described as local
  pure-Rust coverage, without over-claiming pgrx/PG18 callback coverage.
- Whether the mutation target list is a reasonable first critical-module list.
- Whether the lane defaults are conservative enough for report-first burn-in.

## Validation

Packet-local artifacts are in `artifacts/`.

- `bash-n-hardening.log`: `bash -n scripts/hardening.sh` passed.
- `bash-n-install-hardening-tools.log`: `bash -n scripts/install_hardening_tools.sh` passed.
- `make-n-quality-lanes.log`: `make -n` expanded all new quality lane commands.
- `mutants-tool-check.log`: confirms missing `cargo-mutants` fails with setup text.

The long coverage, mutation, and flake sweeps were not run in this slice.
`cargo-mutants` is not installed in this environment.
