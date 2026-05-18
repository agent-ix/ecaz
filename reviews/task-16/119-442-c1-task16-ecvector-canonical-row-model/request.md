# Review Request: C1 Task16 Ecvector Canonical Row Model

Current head at execution: `10495af`

## Context

Task 16's inline-raw measurement arc established that the real serious-lane
lever is the heap row model:

- packet `440` showed persisted `source_raw` helps, but only modestly
- packet `441` showed inline raw-f32 heap storage is the big win
- the previous user-visible row surface was still wrong for that result:
  - indexed column was quantized
  - exact/rerank paths needed a second raw column

This slice straightens that out.

The intended product model is now explicit:

- canonical row column: `ecvector(dim)`
- quantization: index/internal concern
- optional persisted quant artifact: separate sibling type `ecqvector`
- no public `tqvector` SQL surface

## What Landed

### Canonical raw row type

- `sql/bootstrap.sql`
  - `ecvector` is the canonical raw-f32 type
  - `ecvector_ip_ops` is the indexed-column opclass surface
- `src/am/source.rs`
  - indexed-column resolution now distinguishes raw indexed `ecvector`
    from indexed quantized `ecqvector`

### Explicit quantized sibling type

- `sql/bootstrap.sql`
  - `ecqvector` is the sibling quantized artifact type
  - `ecqvector_ip_ops` is available for explicit quantized-artifact tests
- `src/lib.rs`
  - the quantized-artifact fixtures that still intentionally test persisted
    quantized rows now use `ecqvector`

### AM runtime now defaults to the indexed `ecvector` column

- `src/am/build.rs`
  - indexed `ecvector` can be read directly as the build/raw source
  - quantized storage is derived from that raw indexed column during build
- `src/am/insert.rs`
  - default source fallback now uses the indexed `ecvector` column when no
    alternate raw source column is configured
- `src/am/vacuum.rs`
  - the same indexed-column fallback is available in maintenance paths
- `src/am/scan.rs`
  - default `heap_f32` rerank resolves to the indexed `ecvector` column
    instead of requiring a duplicated-row source column

### Old `tqvector` SQL surface removed

- the public `tqvector` SQL type is gone
- the old compatibility helper `encode_to_tqvector(...)` is now also gone
- test SQL now uses the explicit new names:
  - `encode_to_ecvector(...)`
  - `encode_to_ecqvector(...)`

The result is that current head no longer preserves the old SQL-name surface
as a compatibility shim.

## Validation

Green checkpoint on current head:

- `cargo test`
- `bash scripts/run_pgrx_pg17_test.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Notable regression surfaces exercised by that checkpoint:

- `ecvector` text/binary I/O and typmod enforcement
- indexed `ecvector` build / insert / scan / vacuum paths
- indexed `ecqvector` explicit quantized-artifact paths
- default rerank resolution on indexed `ecvector`
- prior task-16 TurboQuant and PqFastScan coverage on current head

## Readout

### 1. The row model now matches the intended product design

Current head no longer asks users to store a quantized main vector column
and then add a second raw column to recover exactness. The canonical row
value is raw `ecvector`.

### 2. Quantization is still supported, but no longer baked into the row type

TurboQuant and PqFastScan keep their quantized index/runtime paths. The
difference is where quantization lives:

- canonical row value: `ecvector`
- explicit quant artifact, if needed: `ecqvector`
- main ANN quantization path: internal to the index/runtime

### 3. Task 16 is unblocked for the final head-to-head measurement phase

This slice does not close task 16 by itself. It lands the product surface
needed for the remaining measurements to be meaningful on the final model:

- rerun the inline serious lane on `ecvector`, not the bytea research surface
- run the requested TurboQuant vs PqFastScan head-to-head on that same
  canonical-row surface
- close the remaining landing checklist items in
  `plan/tasks/16-turboquant-iteration.md`

## Review focus

1. Is the indexed-column fallback resolution now coherent across build,
   insert, scan, and vacuum for `ecvector` vs `ecqvector`?
2. Did any test surface that should still be explicitly quantized get
   accidentally converted from `ecqvector` semantics to `ecvector`?
3. Is removing the last public `encode_to_tqvector(...)` helper the right
   strictness point for this branch?
