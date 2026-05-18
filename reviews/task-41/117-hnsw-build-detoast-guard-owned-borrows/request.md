# Review Request: Task 41 HNSW Build Detoast Guard-Owned Borrows

Code commit: `98380d5c5f97628238bcb2ab33e9c3329cbf2543`

## Summary

This is a local Task 41 invariant #2 slice for HNSW build tqvector input
decoding.

- Adds a local `DetoastedBuildDatum` guard in `src/am/ec_hnsw/build.rs`.
- Moves `pg_detoast_datum_packed` ownership and `pfree` into the guard.
- Keeps the tqvector build path copying into owned `Vec<u8>` before unpacking,
  with the temporary byte borrow derived from the guard.

## Scope

Touched file:

- `src/am/ec_hnsw/build.rs`

No shared detoast abstraction was introduced. This does not touch HNSW source
wrappers, IVF, DiskANN, SPIRE, buffer/snapshot resources, or panic/`pg_guard`
code.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

Validation artifacts are under `artifacts/`.

## Reviewer Focus

- Confirm detoast-copy ownership and `pfree` are tied to
  `DetoastedBuildDatum`.
- Confirm the tqvector build path still copies bytes before unpacking and
  preserves the previous invalid-tqvector error path.
- Confirm this is local to HNSW build invariant #2 work.
