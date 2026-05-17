# Review Request: C1 ADR-030 V2 Shared Source Metric and Live Source Insert

## Context

ADR-032 and task 15 shifted the branch from a grouped experiment into a
first-class two-format design. The prior runtime-adapter checkpoint split
insert/vacuum into shared lifecycles with explicit format seams, but the next
real blocker was still unresolved:

- `build_source_column` indexes build graph topology in source space
- live insert still had no reusable source-column fetch path
- build and scan each carried their own private source-column helper copies
- `aminsert` still hard-rejected every `build_source_column` index before any
  grouped work could reuse that seam

That made both scalar source-backed maintenance and future `PqFastScan`
maintenance harder than they needed to be.

## Problem

There were two concrete issues:

1. source-column heap decoding lived in duplicated local helpers
   - `src/am/build.rs` had one copy
   - `src/am/scan.rs` had another
2. live insert could not follow source-space graph semantics
   - it rejected `build_source_column` indexes outright
   - even scalar source-backed indexes could not accept live inserts, despite
     build already constructing their graph from source vectors

Without fixing that, grouped insert would either add a third source helper copy
or keep pushing source-space maintenance logic into one-off format branches.

## Planned Slice

One narrow checkpoint:

1. centralize source-column resolution, heap fetch, and flat `real[]` / `bytea`
   decoding into one AM-local helper module
2. use that shared source path to remove the scalar live-insert rejection for
   `build_source_column` indexes and route insert-time graph search/backlinks
   through a source-space scorer when source vectors are present

This slice still does **not** implement grouped append or grouped vacuum.

## Implementation

Updated:

- `src/am/source.rs`
- `src/am/mod.rs`
- `src/am/build.rs`
- `src/am/scan.rs`
- `src/am/insert.rs`
- `src/lib.rs`

### 1. Added one shared source helper module

`src/am/source.rs` now owns the reusable source-column plumbing:

- weighted representative averaging
- source-column attnum/type resolution
- tuple-slot datum extraction
- heap row fetch by TID into a reusable slot
- flat `real[]` and `bytea` float4 decoding
- generic negative inner-product scoring on raw source vectors

That replaces the duplicated build-side and scan-side helper copies.

### 2. Build and scan now consume the shared source helpers

`src/am/build.rs` now resolves `build_source_column` through the shared source
module and reuses the shared representative-averaging helper.

`src/am/scan.rs` now resolves grouped heap-rerank source columns, fetches heap
rows, and decodes `real[]` / `bytea` sources through the same shared module
instead of its private helper copy.

This is a code-shape cleanup, not a runtime-behavior change for scan/build.

### 3. Live insert now supports scalar `build_source_column` indexes

`src/am/insert.rs` no longer hard-rejects `build_source_column` indexes.

Instead, when the reloption is present:

- `aminsert` allocates a reusable heap source slot
- fetches the new row’s source vector from the heap using `SnapshotSelf`
- builds the live tuple with `source_vector`
- routes insert-time graph search through `InsertSearchMetric::Source`

That source metric:

- scores new-tuple-to-existing-element comparisons in source space
- reconstructs averaged element representatives across duplicate heap TIDs
- uses the same source-space metric for backlink rewrite selection

So the scalar source-backed graph now keeps one maintenance metric across build
and live insert instead of silently falling back to code-space insert behavior.

### 4. Added pg regression coverage for live source-backed insert

`src/lib.rs` replaces the old rejection test with a success-path regression:

- create a scalar index with `build_source_column = 'source'`
- perform a live insert after index build
- verify the inserted heap TID is present in the index
- verify the new element keeps a persisted neighbor tuple
- verify `inserted_since_rebuild` still advances

## Measurements

No new benchmark or recall measurements in this slice. This is maintenance-path
correctness and architecture groundwork.

## Validation

Passed:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`, and
  `errstart`

## Outcome

This checkpoint does three useful things:

1. source-column maintenance now has one shared helper surface instead of
   duplicated build/scan plumbing
2. scalar `build_source_column` indexes can now accept live inserts
3. insert-time graph maintenance can switch metrics cleanly based on stored
   format/reloption needs, which is a direct prerequisite for grouped
   maintenance too

What it still does **not** do:

- no grouped append path yet
- no grouped vacuum path yet
- no reloption rename/cutover yet (`storage_format`)

## Next Slice

The next clean checkpoint should carry the same source-space maintenance seam
into vacuum repair/finalization so source-backed scalar indexes and future
`PqFastScan` maintenance do not diverge between insert and vacuum.
