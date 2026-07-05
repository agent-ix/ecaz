# Review Request: Task 41 Invariant #2 DiskANN slot Datum lifetime audit

Audit head: `902d924bed5b3f107b8033ed38f7daa9d497dc89`

## Summary

This packet covers the first Phase B target from the invariant #2 strategy:
`src/am/ec_diskann/scan_state.rs`.

The DiskANN scan slot-Datum path does not need a code change in this slice.
`required_slot_datum` has one production caller. That caller fetches the heap
row into the slot, immediately passes the returned Datum into
`ambuild::with_ecvector_datum_slice`, consumes the borrowed vector inside the
closure, clears the slot, and returns only the closure result.

No borrowed `Datum`, `&[u8]`, `&[f32]`, or wrapper backed by
`TupleTableSlot.tts_values` escapes past `ExecClearTuple`.

## Scope

- Audit-only packet; no code change.
- Covered `src/am/ec_diskann/scan_state.rs` and its DiskANN callers.
- Did not touch SPIRE, HNSW, CustomScan, palloc scan-state slices, or
  buffer/page surfaces.

## Evidence

- `artifacts/diskann-slot-callers.log` shows the only DiskANN
  `required_slot_datum` caller and its immediate `with_ecvector_datum_slice`
  consumption.
- `artifacts/scan-state-slot-excerpt.log` captures the slot fetch, clear, and
  Datum read helpers.
- `artifacts/routine-rerank-slot-excerpt.log` captures the caller lifetime
  shape.
- `artifacts/git-status.log` confirms no source change was present for this
  audit packet.

## Validation

No tests were run because this is an audit-only packet with no code change.

## Reviewer Focus

- Confirm the returned Datum is consumed before the slot is cleared or reused.
- Confirm `with_ecvector_datum_slice` closure-scopes the borrowed vector so no
  slot-backed borrow can escape.
- Confirm Phase B can mark DiskANN scan slot-Datum reads as audited.
