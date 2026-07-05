# Task 64 Review Request: HNSW Codec Build Sizing

## Summary

This checkpoint moves HNSW build-time tuple fit checks behind the local codec
adapter added in packet 001. It is still adapter-only and does not add RaBitQ.

Code commit under review:

- `1eac01c43f81238656ea063f15e2ec019ad7a6ed` - Move HNSW build sizing into codec adapter

## Changes

- Added `HnswStorageCodec::build_tuple_fits_on_page`.
- Routed `BuildState::push` through the codec adapter for the first tuple's
  page-fit validation.
- Preserved the existing format behavior:
  - TurboQuant checks hot tuple plus cold rerank tuple storage.
  - PqFastScan keeps the previous encoded element tuple sizing check.

## Task 63 Handoff

RaBitQ should add its build hot/cold sizing rule in
`HnswStorageCodec::build_tuple_fits_on_page` before adding build flush support.
That keeps tuple-size validation aligned with metadata identity and graph
storage layout.

## Validation

- `cargo check -q --lib` passed.

See `artifacts/manifest.md` for command metadata.
