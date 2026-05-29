# Task 63 Review Request: HNSW RaBitQ Build Payload

## Scope

This checkpoint wires the HNSW build flush path for `storage_format = 'rabitq'`.
It consumes the shared `RaBitQQuantizer`, writes RaBitQ search codes into the
grouped hot tuple payload, and keeps the existing scalar-quantized cold rerank
payload for exact rerank compatibility.

Code commit under review:

- `9b08063814d8deef7b0b4eb6f7cb9519d812e1c0` - Add HNSW RaBitQ build payload

## Implementation Notes

- `flush_build_state_with_timing` now routes RaBitQ builds through a dedicated
  build flush path instead of failing at metadata recognition.
- RaBitQ search payloads are encoded from the source vector using
  `RaBitQQuantizer::cached_seeded_srht_bits` and the index seed.
- The staged page chain reuses HNSW's grouped hot tuple layout with no binary
  sidecar and a cold `TqRerankTuple` for TurboQuant-compatible rerank payloads.
- Builds fail closed when no source vector is available, so `tqvector` inputs
  must provide `build_source_column` until a durable source-vector strategy is
  added.

## Deliberate Limits

This is not yet a full RaBitQ HNSW runtime. The graph storage descriptor, scan
scoring, insert path, vacuum path, and PG18 SQL smoke coverage are still pending
Task 63 slices.

## Validation

- `cargo check -q --lib` passed.

Artifact manifest:

- `artifacts/manifest.md`
