# Task 63 Review Request: HNSW RaBitQ Storage Identity

## Summary

This checkpoint adds the HNSW RaBitQ storage identity and metadata
discriminator, but intentionally does not claim build/scan payload support yet.

Code commit under review:

- `8817afe147cc49ea69733d50d1280c38097256d0` - Add HNSW RaBitQ storage identity

## Changes

- Added `rabitq` to the shared HNSW storage-format family used by
  `ec_hnsw` reloptions.
- Added HNSW metadata constants and decode support for:
  - `INDEX_FORMAT_V4_RABITQ`;
  - `SearchCodecKind::RaBitQ`;
  - `GraphStorageFormat::RaBitQ`.
- Extended `HnswStorageCodec` to map RaBitQ reloptions and metadata.
- Added initial empty RaBitQ metadata with:
  - no binary sidecar flag;
  - cold rerank payload flag;
  - RaBitQ search codec;
  - default quant bit width recorded in search fields.
- Added explicit build/graph rejection messages for RaBitQ paths whose payload
  support is still pending.

## Current Limitation

`CREATE INDEX ... USING ec_hnsw WITH (storage_format = 'rabitq')` is not
expected to work yet. This packet only lands identity, metadata, and adapter
extension points. The next implementation packet must add the actual RaBitQ
build payload and graph storage descriptor support.

## Validation

- `cargo check -q --lib` passed.

See `artifacts/manifest.md` for command metadata.
