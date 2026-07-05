# Review Request: Task 41 Invariant #2 ecvector typmod detoast guard

Code commit: `93610edafe311ee8d8ee98f3518590efcd67a581`

## Summary

This local slice closes the remaining non-AM open-coded detoast lifetime in
`src/lib.rs`.

`ecvector_typmod_in` now keeps the packed typmod array borrow behind a local
`DetoastedTypmodArray` guard. The guard records whether
`pg_detoast_datum_packed` returned a copied varlena pointer and owns the
matching `pfree` in `Drop`, so the raw `ArrayType` pointer cannot outlive the
PostgreSQL-owned or guard-owned storage by construction.

## Scope

- Changed `src/lib.rs` only.
- Preserved existing typmod validation behavior and error text.
- Did not introduce a shared detoast abstraction; this is a local guard for
  the extension SQL typmod parser.
- Did not touch AM page, buffer, scan-state, or callback lifetime surfaces.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

The PG18 cargo check completed successfully with the pre-existing unused import
warning in `src/am/mod.rs`. No pgrx runtime tests were run for this narrow
local lifetime refactor.

## Artifacts

- `artifacts/fmt-check.log`
- `artifacts/cargo-check-pg18.log`
- `artifacts/git-diff-check.log`
- `artifacts/code-diff-stat.log`
- `artifacts/manifest.md`

## Reviewer Focus

- Confirm the `ArrayType` pointer use remains fully inside the guard lifetime.
- Confirm the copied detoast case is freed exactly once, and the non-copied
  case is never freed by this guard.
- Confirm this local slice does not overlap with the AM-local detoast guard
  packets or the pending buffer/page work for invariant #3.
