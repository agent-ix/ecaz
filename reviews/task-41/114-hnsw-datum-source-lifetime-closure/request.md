# Review Request: Task 41 HNSW Datum Source Lifetime Closure

Code commit: `e5dbe022da79d969757cdccd059e841695aa3493`

## Summary

This slice starts Task 41 invariant #2, memory-context lifetime safety, on the
HNSW/IVF heap-source decoder path.

- Adds explicit lifetimes to `FlatFloat4ArrayRef`, `FlatFloat4VarlenaRef`, and
  `FlatFloat4SourceRef`.
- Makes the raw `from_datum` constructors private to `source.rs`.
- Replaces return-a-wrapper APIs with higher-ranked closure helpers:
  `with_flat_float4_source_from_datum`, `with_source_from_heap_row`, and
  `with_indexed_ecvector_from_slot`.
- Migrates HNSW build/insert/vacuum/scan and IVF heap-rerank callers so
  Datum-backed slices are copied or scored inside the borrow closure.

The intended invariant is that callers can no longer return or store a
PG-memory-backed float slice wrapper from these HNSW source helper APIs. They
must consume it immediately, before tuple-slot clearing or detoast-copy free.

## Scope

Touched files:

- `src/am/ec_hnsw/source.rs`
- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/insert.rs`
- `src/am/ec_hnsw/scan.rs`
- `src/am/ec_hnsw/vacuum.rs`
- `src/am/ec_ivf/scan.rs`

This does not touch Task 41 invariant #1 panic/`pg_guard` inventory, and does
not alter the invariant #3 buffer/snapshot/release wrapper track.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

Validation artifacts are under `artifacts/`.

## Reviewer Focus

- Confirm the higher-ranked closure API prevents borrowed Datum source wrappers
  from escaping the helper call.
- Confirm the migrated callers preserve previous behavior: copy-to-`Vec`,
  score immediately, then clear tuple slots where they did before.
- Confirm this is a clean invariant #2 slice and does not overlap the resource
  release work another agent is doing for invariant #3.
