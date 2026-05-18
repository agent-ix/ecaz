# Review Request: Task 41 Invariant #2 SPIRE scan slot Datum lifetime audit

Audit head: `4a9e63f524b26919fd6c4ecf664c515266704b15`

## Summary

This packet covers the second Phase B target from the invariant #2 strategy:
`src/am/ec_spire/scan/relation.rs`.

The SPIRE heap-rerank slot-Datum path does not need a code change in this
slice. `required_slot_datum` is local to the file, and its returned Datum is
immediately converted by `indexed_vector_datum_to_source_vector` into owned
bytes and then an owned `Vec<f32>`. The tuple slot is cleared only after that
conversion completes.

No `Datum`, detoasted byte slice, source-vector slice, or wrapper backed by
`TupleTableSlot.tts_values` escapes past `ExecClearTuple`.

## Scope

- Audit-only packet; no code change.
- Covered `src/am/ec_spire/scan/relation.rs`.
- Did not touch DiskANN, HNSW, CustomScan tuple output, palloc scan-state
  slices, or buffer/page surfaces.
- Existing unrelated comparator-script worktree changes are outside this
  packet.

## Evidence

- `artifacts/spire-scan-slot-callers.log` shows the local slot-Datum call chain.
- `artifacts/spire-scan-slot-excerpt.log` captures fetch, required Datum read,
  owned conversion, and slot clear.
- `artifacts/spire-detoast-copy-excerpt.log` captures the guard-owned detoast
  copy path that returns owned bytes.
- `artifacts/git-status.log` records the audit head worktree context.

## Validation

No tests were run because this is an audit-only packet with no code change.

## Reviewer Focus

- Confirm the slot-backed Datum is converted to owned data before slot clear.
- Confirm the detoasted varlena borrow is guard-owned and copied to `Vec<u8>`
  before the guard drops.
- Confirm Phase B can mark SPIRE scan relation slot-Datum reads as audited.
