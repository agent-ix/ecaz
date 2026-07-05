# Review Request: Task 41 DiskANN Build Detoast Guard-Owned Borrows

Code commit: `54d98fea6031673a9bc14bff8b7d9b7024cc35c0`

## Summary

This is a local Task 41 invariant #2 slice for DiskANN build ecvector decoding.

- Adds a local `DetoastedEcvectorDatum` guard in
  `src/am/ec_diskann/ambuild.rs`.
- Moves `pg_detoast_datum` ownership and `pfree` into the guard.
- Keeps `with_ecvector_datum_slice` as the caller-facing scoped-borrow API, now
  with detoast-copy lifetime owned by the wrapper that backs the slice.

## Scope

Touched file:

- `src/am/ec_diskann/ambuild.rs`

No shared detoast abstraction was introduced. This does not touch HNSW, IVF,
SPIRE, buffer/snapshot resources, or panic/`pg_guard` code.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

Validation artifacts are under `artifacts/`.

## Reviewer Focus

- Confirm detoast-copy ownership and `pfree` are tied to
  `DetoastedEcvectorDatum`.
- Confirm `with_ecvector_datum_slice` still scopes the `&[f32]` borrow to its
  closure and preserves existing validation errors.
- Confirm this is local to DiskANN build invariant #2 work.
