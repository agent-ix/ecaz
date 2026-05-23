# Task 50 Review Request: SPIRE Object Tuple Visitor

## Summary

This slice rolls the `LockedBufferGuard::visit_tuple_bytes` contract from packet
362 through SPIRE object tuple reads.

Changes:

- `src/am/ec_spire/page.rs` now uses the locked-buffer visitor for
  `with_pinned_object_tuple` and `scan_object_tuples`.
- Removed the SPIRE-only immutable raw page tuple visitor and its result enum.
- Kept same-length rewrite on the WAL/exclusive page path, but changed it to a
  mutable visitor and `copy_from_slice` instead of mutating through an immutable
  byte slice.

The direct `unsafe { ... }` count moved from `1213` to `1210`.

Code commit: `e1bf25f1c0311c447d92fdb44535a96c561a4d66`

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed. It still reports the known pre-existing SPIRE DML unused-import
  warning in `src/am/mod.rs`.
- `git diff --check` passed.
- Raw-boundary guard scan produced no matches. The `rg` command exits 1 because
  it found no rows.
- SPIRE object tuple helper scan confirms immutable object reads use the shared
  locked-buffer boundary; the remaining SPIRE page tuple boundary is the
  WAL/exclusive mutable rewrite helper.
- Unsafe ledger check passed for `1210` current unsafe rows.

Artifacts are recorded in `artifacts/manifest.md`.
