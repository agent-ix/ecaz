# Task 50 Review Request: SPIRE Debug Safe Test API

## Summary

This slice makes SPIRE test/debug helpers safe at their public call sites when
the helper already opens and owns the required PostgreSQL relation guard.

The actual PostgreSQL page/store unsafety remains inside
`ec_spire::coordinator::debug` and `ec_spire::vacuum`, with explicit internal
unsafe blocks. The many pg_test callers now call the debug helpers directly
instead of wrapping each call in local unsafe blocks.

## Files Changed

- `src/am/ec_spire/coordinator/debug.rs`
- `src/am/ec_spire/vacuum/mod.rs`
- SPIRE pg_test callers under `src/tests/`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2402 -> 2283`.
- Touched direct unsafe blocks: `176 -> 70`.
- Changed files: `26`.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_spire/coordinator/debug.rs src/am/ec_spire/vacuum/mod.rs`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`

Known pre-existing warnings are unchanged:

- normal `cargo check`: SPIRE DML test re-export unused-import warning in
  `src/am/mod.rs`
- `pg_test` no-run: Hadamard test-only helper dead-code warnings

## Artifacts

- `artifacts/manifest.md`
- `artifacts/unsafe-counts.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pg-test-no-run.log`
