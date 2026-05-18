# Review Request: Task 41 amvalidate pg_guard coverage

## Summary

Task 41 invariant #1 slice for access-method validation callbacks.

This adds `#[pg_guard]` to the `amvalidate` callbacks in:

- `src/am/ec_hnsw/routine.rs`
- `src/am/ec_ivf/routine.rs`
- `src/am/ec_diskann/routine.rs`
- `src/am/ec_spire/routine.rs`

Code commit: `1f578271`

## Safety Effect

- Makes the four `IndexAmRoutine::amvalidate` callbacks explicit pgrx-guarded
  PostgreSQL callback boundaries.
- Keeps the existing callback behavior unchanged: each callback still returns
  `true`.
- Updates the unsafe comment baseline line map after adding one line above the
  existing DiskANN unsafe baseline region; the entry count remains `3701`.

## Review Focus

- Confirm these four callbacks are real PostgreSQL callback entry points through
  `amroutine.amvalidate = Some(...)`.
- Confirm `#[pg_guard]` is accepted on non-`no_mangle` callback functions used
  by an `IndexAmRoutine` function pointer.
- Confirm no invariant #2 datum-lifetime code is touched by this packet.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
