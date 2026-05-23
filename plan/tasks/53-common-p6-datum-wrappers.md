# Task 53: Common P6 — Datum / Varlena / EcVector Typed Wrappers

Status: **proposed** — supersedes the §P6 disposition in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`
and the source.rs structural-ceiling rationale in
`reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md`.
Second Phase-1 lane in the post-Task-50 hardening sequence.

## Why

Multiple AMs share the same documented residual unsafe pattern at their
datum-handling boundaries. The 448 closeout records HNSW's `source.rs`
ceiling at this pattern explicitly:

> 2. `FromDatum` / detoast / varlena bytes (lines 503-660):
>    `DetoastedVarlena::plain_from_datum`,
>    `DetoastedFloat4Datum::from_datum`, raw `pg_sys::ArrayType`
>    header reads, `flat_array_dims_ptr` / `flat_array_data_offset`,
>    `from_raw_parts` over the data slice. ... The boundary is the PG
>    datum ABI; these are precisely the blocks **program P6** ("Datum,
>    Varlena, Vector, And Quantized Payload Contracts") plans to
>    encapsulate via `FlatFloat4Source<'a>` and `EcVectorDatum<'a>`
>    typed wrappers.

The same pattern lives in `src/am/ec_ivf/scan.rs`,
`src/am/ec_diskann/scan_state.rs`, `src/am/ec_diskann/insert.rs`, and
`src/am/ec_spire/insert.rs`'s vector-extraction paths. P6 lifts those
once into typed wrappers in `src/am/common/`; consumers across all
four AMs become safe call sites.

## Non-Goals

- Do not refactor RaBitQ scoring math. P6 is about payload boundaries,
  not the quantizer surface (which is Task 51's domain).
- Do not change on-disk varlena format. Wrappers are read-side ABI
  shims, not format changes.
- Do not touch SIMD `#[target_feature]` intrinsic functions — those
  remain `unsafe fn` per Rust's language-level requirement and are not
  in P6 scope.
- Do not migrate IVF or SPIRE consumer call sites in this task. They
  will consume the wrappers under Tasks 57 (IVF, gated on Task 51) and
  56 (SPIRE, gated on optimization stability).

## Scope

Add typed wrappers in a new module `src/am/common/datum.rs` (or
sibling files if the surface deserves separation):

1. **`DetoastedVarlena<'a>`** — typed RAII view over a detoasted
   varlena. `unsafe fn from_datum(datum)` is the only unsafe surface;
   `as_bytes() -> &[u8]`, `as_typed_slice::<T>() -> &[T]`, and
   `Drop` (releases the toast pointer if owned) are safe.
2. **`FlatFloat4Source<'a>`** — typed wrapper over a flat float4
   PostgreSQL array. Encapsulates `pg_sys::ArrayType` header read,
   dimension/offset arithmetic, and the `from_raw_parts` over the data
   slice. Safe `len()`, `as_slice() -> &[f32]`, `dims() -> usize`.
3. **`EcVectorDatum<'a>`** — typed wrapper over the
   `Datum -> EcVector` extraction used by HNSW source.rs, IVF
   scan.rs, DiskANN insert.rs, and SPIRE insert.rs. One `unsafe fn
   from_datum`; safe `view() -> EcVectorView<'a>` accessor that does
   the FromDatum + detoast + flat-array boundary in one place.
4. **`AttnumLookup<'a>`** — typed wrapper over the
   `pg_sys::get_attnum(rel, attname)` catalog boundary. Encapsulates
   the unsafe extern call into a safe `lookup(rel, attname) ->
   Option<AttrNumber>` API.

Each wrapper records its PG-datum lifetime invariant in its
constructor doc, same pattern as the slice-447 P8 module.

## Migration Targets

This task migrates **HNSW only** as the validating consumer. SPIRE /
IVF / DiskANN consumer migrations belong to their own subsystem tasks
(Tasks 55/56/57).

| File | Surface | Expected block delta |
| --- | --- | ---: |
| `src/am/ec_hnsw/source.rs` (29 currently) | `DetoastedVarlena`, `DetoastedFloat4Datum::from_datum`, `pg_sys::ArrayType` header reads, `flat_array_dims_ptr` / `flat_array_data_offset`, `from_raw_parts` data slice, `pg_sys::get_attnum` | -15 to -20 |

**Target**: `source.rs` 29 → ≤ 14 (-52% or better).

## Techniques

- Single `unsafe fn` constructor per wrapper. All read methods safe.
- Tie lifetimes to caller-known live regions (e.g. detoasted varlena
  outlives the wrapper; PG-arena scope is the borrow lifetime).
- For RAII-released toast pointers, `Drop` performs `pfree` only when
  the wrapper owns the detoasted allocation (not when it borrows).

## Slice and Packet Rules

Same as Tasks 50 / 52. Specifically:

- Each packet must report `unsafe { ... }` block count before / after
  for every touched file, plus `src/` total.
- Wrapper-side blocks in `src/am/common/datum.rs` are counted but
  recorded as the intended category shift (per P6's disposition).
- HNSW source.rs migration may land in one or two slices; further
  reductions wait on the SPIRE / IVF / DiskANN tasks consuming the
  same wrappers.

## Performance Gate

`source.rs` is on the scoring hot path via
`indexed_vector_from_datum` and friends. The wrapper must inline
through to the same machine code as the open-coded version.

Required evidence per slice that touches `source.rs`:

- `ecaz bench latency` + `ecaz bench recall` on the post-Task-50 M5
  baseline corpus (`benchmarks/task-50-m5-hnsw-baseline/`) at the
  same prefixes and sweep, before/after.
- Per-row storage from `ecaz bench storage` must not change (the
  wrappers are read-side, not format).

Acceptance: regression tolerance is the same as Tasks 50/52.

## Validation

- `cargo fmt --all`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- focused `cargo pgrx test pg18 ec_hnsw::source` when datum-extraction
  behavior could plausibly drift
- direct unsafe-block count per touched file
- `src/` total snapshot

## Exit Criteria

Task closes when:

- The four typed wrappers above exist in `src/am/common/datum.rs`.
- `src/am/ec_hnsw/source.rs` block count ≤ 14.
- HNSW recall + QPS + per-row storage show no regression vs the
  post-Task-50 baseline.
- A closing summary packet records:
  - per-file before/after for `source.rs`;
  - the `src/am/common/datum.rs` wrapper surface;
  - the `src/` total block count change;
  - explicit handoff list naming each SPIRE / IVF / DiskANN consumer
    site that the new wrappers will absorb under Tasks 55/56/57.

## Coordination

- Phase-1 lane — runs after Task 52 (so the `dsm.rs` + `datum.rs`
  module pattern is consistent) and before Task 54 (P3 page/WAL).
- HNSW-only consumer migration. Tasks 55/56/57 migrate their own AM's
  consumers once they open.
- Coordinate with Task 51 (IVF RaBitQ optimization): no overlap
  expected since IVF datum-handling is not refactored here, only
  named in the handoff list.
- Reviewer scope-lock: HNSW-only consumer migration on this branch.

## Cross-References

- Supersedes `reviews/task-50/030-comprehensive-unsafe-burndown-plan`
  §P6 disposition.
- Closes the `source.rs` ceiling documented in
  `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md`
  §"`source.rs` ceiling".
- Bench gate consumes
  `benchmarks/task-50-m5-hnsw-baseline/manifest.md` as the pre-state.
