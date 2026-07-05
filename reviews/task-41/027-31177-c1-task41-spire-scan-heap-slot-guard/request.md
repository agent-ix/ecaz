# Review Request: Task 41 SPIRE Scan Heap Slot Guard

Code commit: `3582fd276ea7175cf209d1f97185b4ce0cb695d6`

## Summary

This checkpoint wraps the heap tuple slot used by SPIRE heap rerank scan
candidate preparation in `src/am/ec_spire/scan/relation.rs`.

- Adds `HeapTupleSlot`, which owns the `MakeSingleTupleTableSlot` result.
- Removes the manual `ExecDropSingleTupleTableSlot` call after candidate
  preparation.
- Keeps the slot live across all heap source-vector loads through a borrowed
  raw pointer.

## Safety Delta

- Baseline entries: `4315` -> `4313`.
- `src/am/ec_spire/scan/relation.rs`: `36` -> `34`.
- This is a behavioral safety improvement: errors during candidate
  preparation now still drop the slot through RAII.

## Reviewer Focus

- Confirm `HeapTupleSlot` drop happens after the last closure use of
  `slot.as_ptr()`.
- Confirm no returned value from candidate preparation borrows from the slot.
- Confirm the slot is still cleared by `load_indexed_source_vector_from_heap_row`
  per row before the final slot drop.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see
`artifacts/manifest.md`.
