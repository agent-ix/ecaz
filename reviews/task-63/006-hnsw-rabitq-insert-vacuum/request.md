# Task 63 Review Request: HNSW RaBitQ Insert and Vacuum

## Scope

This checkpoint completes the non-scan runtime policy for HNSW RaBitQ by adding
live insert and vacuum support instead of leaving those paths fail-closed.

Code commit under review:

- `a9d4930bc0e5be1ab4115d474af5f96416176ea8` - Support HNSW RaBitQ insert and vacuum

## Implementation Notes

- Empty RaBitQ indexes now bootstrap through the RaBitQ build flush path on
  first insert.
- Live inserts derive a RaBitQ search code from raw source data using the shared
  `RaBitQQuantizer` and write grouped hot + cold rerank tuples.
- Duplicate coalescing reuses the grouped hot tuple path and compares the cold
  scalar-quantized rerank payload.
- Vacuum resolves RaBitQ to the grouped hot/cold graph descriptor, reusing the
  existing cleanup, repair, and finalization machinery.

## Validation

- `cargo check -q --lib` passed.
- `cargo test -q --lib graph_storage_descriptor_uses_rabitq_code_len_for_v4_metadata --no-run`
  passed. The command emitted existing test warning noise about unnecessary
  `unsafe` blocks and unused Hadamard test helpers.

Artifact manifest:

- `artifacts/manifest.md`
