# Review Request: Task 41 HNSW Detoast Guard-Owned Borrows

Code commit: `ffe7338d84160b743e751e2c51e64a5b3e650832`

## Summary

This is a local Task 41 invariant #2 follow-up for `src/am/ec_hnsw/source.rs`.

- Introduces `DetoastedFloat4Datum` inside the HNSW source module.
- Moves detoast-copy ownership and `pfree` into that local guard.
- Makes `FlatFloat4ArrayRef` and `FlatFloat4VarlenaRef` hold the guard instead
  of carrying raw `(ptr, owned)` fields directly.

The previous packet made callers consume Datum-backed float views through
higher-ranked closure helpers. This packet keeps the remaining detoasted
varlena lifetime local to the wrapper itself, so the raw byte slice returned by
`varlena_to_byte_slice` is derived from a field owned by the wrapper and freed
only when the wrapper drops.

## Scope

Touched file:

- `src/am/ec_hnsw/source.rs`

No shared abstraction was introduced, and no IVF, DiskANN, SPIRE, buffer,
snapshot, or panic/`pg_guard` code was changed.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

Validation artifacts are under `artifacts/`.

## Reviewer Focus

- Confirm `FlatFloat4ArrayRef` and `FlatFloat4VarlenaRef` now own the detoast
  guard that backs their returned slices.
- Confirm the local guard preserves prior behavior: borrowed Datums are not
  freed, detoast copies are freed on drop, and validation errors still report
  through the same `ec_hnsw` labels.
- Confirm this slice stays local to HNSW invariant #2 and does not overlap the
  other Task 41 invariant tracks.
