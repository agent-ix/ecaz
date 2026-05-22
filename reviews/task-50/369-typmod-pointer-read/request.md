# Task 50 Review Request: Typmod Pointer Read

## Summary

This slice tightens the root `ecvector` typmod input boundary.

Changes:

- `src/lib.rs` now performs `ArrayGetIntegerTypmods`, the single-element count
  check, the null check, and the first-element pointer read inside one checked
  boundary.
- The typmod pointer read now explicitly rejects a null pointer even if
  PostgreSQL reports one typmod.

The direct `unsafe { ... }` count moved from `1199` to `1198`.

Code commit: `e2279726d5cd24b5132a4ecabee2e57e26cafd27`

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed. It still reports the known pre-existing SPIRE DML unused-import
  warning in `src/am/mod.rs`.
- `git diff --check` passed.
- Raw-boundary guard scan produced no matches. The `rg` command exits 1 because
  it found no rows.
- Typmod pointer scan confirms the typmod pointer read is now guarded by both
  count and null checks in one boundary.
- Unsafe ledger check passed for `1198` current unsafe rows.

Artifacts are recorded in `artifacts/manifest.md`.
