# Task 63 Review Request: HNSW RaBitQ Graph Descriptor

## Scope

This checkpoint adds the runtime graph descriptor for HNSW RaBitQ metadata. It
lets the graph layer decode RaBitQ indexes as grouped hot tuples with a RaBitQ
search-code payload and scalar-quantized cold rerank payload.

Code commit under review:

- `b5aa7766d8e30abebb6127bcc192b4c560e13c0a` - Add HNSW RaBitQ graph descriptor

## Implementation Notes

- Added `GraphStorageDescriptor::RaBitQ` with metadata validation for:
  - `SearchCodecKind::RaBitQ`;
  - no binary sidecar;
  - no grouped PQ codebook chain;
  - no grouped search subvector shape;
  - cold scalar rerank payload.
- Computes RaBitQ hot search-code width through
  `quant::rabitq::code_len_for(dimensions, search_bits)`.
- Reuses the existing grouped hot/cold tuple readers for exact graph element
  loads and debug tuple-tag selection.
- Keeps insert and vacuum fail-closed for RaBitQ until their dedicated Task 63
  slices land.

## Deliberate Limits

This still does not implement RaBitQ traversal scoring. Scan code can identify
the descriptor, but grouped PQ scoring paths remain scoped to PqFastScan.

## Validation

- `cargo check -q --lib` passed.
- `cargo test -q --lib graph_storage_descriptor_uses_rabitq_code_len_for_v4_metadata --no-run`
  passed. The command emitted existing test warning noise about unnecessary
  `unsafe` blocks and unused Hadamard test helpers.

Artifact manifest:

- `artifacts/manifest.md`
