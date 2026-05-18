# Review Request: Spec FR-007 Missing Gamma Field

Scope:
- `spec/functional/FR-007-hnsw-page-layout.md`
- `spec/adr/ADR-013-persist-gamma-in-element-tuples.md`

## Problem

ADR-013 added `gamma: f32` to `TqElementTuple`, but FR-007's field table was never updated.

FR-007 currently lists the element tuple fields as:

| Field | Type | Description |
|---|---|---|
| type | u8 | `0x01` (ELEMENT) |
| level | u8 | HNSW layer |
| deleted | bool | Soft-delete flag |
| heaptids | [ItemPointerData; 10] | Inline heap TID array |
| heaptid_count | u8 | Valid count |
| neighbortid | ItemPointerData | Pointer to neighbor tuple |
| code | [u8; code_len] | Quantized code bytes |

The actual encoding (page.rs:173-188) inserts `gamma: f32` (4 bytes LE) between `heaptid_count`
and `neighbortid`.

FR-007's prose section on duplicate semantics says:
> "gamma remains recoverable from representative heap rows until a future page-layout revision
> persists it in-page"

This is stale — gamma IS persisted in-page now.

## Suggested Fix

1. Add `gamma` row to the FR-007 element tuple table between `heaptid_count` and `neighbortid`
2. Update the duplicate-semantics prose to reflect that gamma is now persisted
3. Update the storage density calculation (~842 bytes already accounts for gamma, but the
   field-by-field breakdown should list it explicitly: `1 + 1 + 1 + 60 + 1 + 4 + 6 + 772 = 846`)

No code changes needed — this is a spec-only update.
