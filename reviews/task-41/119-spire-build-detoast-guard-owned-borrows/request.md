# Review Request: Task 41 SPIRE Build Detoast Guard-Owned Borrows

Code commit: `75130cb30d331a0873043be2702c7df9a59e6156`

## Summary

This is a local Task 41 invariant #2 slice for SPIRE build vector decoding.

- Adds a local `DetoastedBuildDatum` guard in
  `src/am/ec_spire/build/tuples.rs`.
- Moves `pg_detoast_datum_packed` ownership and `pfree` into the guard.
- Keeps `detoasted_varlena_bytes` returning owned `Vec<u8>`, with the
  temporary byte borrow derived from the local guard.

## Scope

Touched file:

- `src/am/ec_spire/build/tuples.rs`

No shared detoast abstraction was introduced. This does not touch SPIRE scan,
HNSW, IVF, DiskANN, buffer/snapshot resources, or panic/`pg_guard` code.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

Validation artifacts are under `artifacts/`.

## Reviewer Focus

- Confirm detoast-copy ownership and `pfree` are tied to
  `DetoastedBuildDatum`.
- Confirm `detoasted_varlena_bytes` still returns owned bytes and preserves the
  previous `ec_spire could not detoast {label}` error.
- Confirm this stays local to SPIRE build invariant #2 work.
