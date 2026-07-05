# Review Request: Task 41 SPIRE Scan Detoast Guard-Owned Borrows

Code commit: `8337a6e36efd4eaeb8fad782af2753aa799da65f`

## Summary

This is a local Task 41 invariant #2 slice for SPIRE heap-rerank scan decoding.

- Adds a local `DetoastedScanDatum` guard in
  `src/am/ec_spire/scan/relation.rs`.
- Moves `pg_detoast_datum_packed` ownership and `pfree` into the guard.
- Keeps `detoasted_varlena_bytes` returning `Result<Vec<u8>, String>`, with the
  temporary byte borrow derived from the local guard.

## Scope

Touched file:

- `src/am/ec_spire/scan/relation.rs`

No shared detoast abstraction was introduced. This does not touch SPIRE build,
HNSW, IVF, DiskANN, buffer/snapshot resources, or panic/`pg_guard` code.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

Validation artifacts are under `artifacts/`.

## Reviewer Focus

- Confirm detoast-copy ownership and `pfree` are tied to `DetoastedScanDatum`.
- Confirm the `Result` error strings are preserved for NULL and failed detoast
  cases.
- Confirm this stays local to SPIRE scan invariant #2 work.
