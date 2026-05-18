# Review Request: Task 41 IVF Build Detoast Guard-Owned Borrows

Code commit: `f934b5d31eb6fda9a550b03698cd9fdfb0f6574a`

## Summary

This is a local Task 41 invariant #2 slice for IVF build input decoding.

- Adds a local `DetoastedBuildDatum` guard in `src/am/ec_ivf/build.rs`.
- Moves `pg_detoast_datum_packed` ownership and `pfree` into the guard.
- Keeps `detoasted_varlena_bytes` returning owned `Vec<u8>`, but now the
  temporary byte borrow is derived from a guard and cannot outlive it.

## Scope

Touched file:

- `src/am/ec_ivf/build.rs`

No shared detoast abstraction was introduced. This does not touch HNSW,
DiskANN, SPIRE, buffer/snapshot resources, or panic/`pg_guard` code.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

Validation artifacts are under `artifacts/`.

## Reviewer Focus

- Confirm detoast-copy ownership and `pfree` are now tied to
  `DetoastedBuildDatum`.
- Confirm `detoasted_varlena_bytes` still returns owned bytes and preserves the
  previous `ec_ivf could not detoast {label}` error.
- Confirm this stays local to IVF build invariant #2 work.
