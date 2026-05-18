# Review Request: Task 41 Invariant #2 HNSW source slot Datum lifetime audit

Audit head: `2539dba25b72e7e2497a579c912c82e0fa560c30`

## Summary

This packet covers the HNSW source-helper portion of Phase B from the invariant
#2 strategy.

The HNSW slot-Datum reads are already bounded by the closure helpers from
packet 114 or copied into owned build state before slot reuse:

- `with_source_from_heap_row` reads a slot Datum and immediately passes it to
  the higher-ranked `with_flat_float4_source_from_datum` closure.
- insert and vacuum callers clear the slot only after that closure returns.
- grouped heap rerank computes a scalar score inside the same closure and then
  clears the slot.
- build-with-source reads the indexed and source Datums from the live scan
  slot, copies the source vector to `Vec<f32>`, then builds an owned
  `BuildTuple` before the next `heap_getnextslot` reuse.

No `Datum`, detoasted byte slice, source-vector slice, or wrapper backed by
`TupleTableSlot.tts_values` escapes past `ExecClearTuple` or heap-scan slot
reuse.

## Scope

- Audit-only packet; no code change.
- Covered HNSW source slot-Datum helpers and their production callers.
- Did not touch DiskANN, SPIRE, CustomScan tuple output, palloc scan-state
  slices, or buffer/page surfaces.

## Evidence

- `artifacts/hnsw-slot-callers.log` lists the HNSW source slot-Datum call
  sites.
- `artifacts/source-slot-helper-excerpt.log` captures the raw slot Datum read
  helper.
- `artifacts/source-closure-helper-excerpt.log` captures the HRTB closure
  helpers.
- `artifacts/build-source-slot-excerpt.log` captures the build scan path that
  copies slot-backed vector data before slot reuse.
- `artifacts/scan-rerank-slot-excerpt.log` captures the grouped heap-rerank
  path that computes a scalar before clearing the slot.

## Validation

No tests were run because this is an audit-only packet with no code change.

## Reviewer Focus

- Confirm every HNSW source slot-Datum read is copied, scalarized, or
  HRTB-closure-scoped before slot clear or reuse.
- Confirm `build_source_column` does not carry `vector_datum` or
  `source_datum` beyond construction of the owned `BuildTuple`.
- Confirm Phase B can mark HNSW source slot-Datum reads as audited.
