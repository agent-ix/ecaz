# Task 64 Review Request: HNSW Codec Adapter Foundation

## Summary

This checkpoint starts Task 64 by adding an HNSW-local storage codec adapter for
the existing `turboquant` and `pq_fastscan` formats. It does not add RaBitQ yet.

Code commit under review:

- `556a11637a1673c224b36e19ca5bdd913d0651ec` - Add HNSW storage codec adapter

## Inventory

Current HNSW format coupling is concentrated in these areas:

- Reloption identity: `options::StorageFormat` currently names `turboquant` and
  `pq_fastscan`.
- Metadata identity: `page::MetadataPage` maps v1/v3 to TurboQuant and v2 to
  PqFastScan.
- Graph storage: `graph::GraphStorageDescriptor` selects tuple layout and
  validates relation reloptions against on-disk metadata.
- Build: `BuildState::initial_metadata` and flush output construction choose
  metadata and payload layout by storage format.
- Insert/vacuum: existing `InsertFormatAdapter` and `VacuumFormatAdapter` keep
  graph mutation shared while dispatching tuple append/read/retention by
  descriptor.
- Scan: grouped traversal and rerank paths branch on `GraphStorageDescriptor`
  and prepared grouped/TurboQuant query state.

## Adapter Shape

The new `src/am/ec_hnsw/codec.rs` introduces `HnswStorageCodec` as the
format-identity layer between reloptions/metadata and the lower-level graph
descriptor. This slice routes:

- reloption format -> codec;
- metadata format -> codec;
- codec -> storage format name;
- codec -> initial empty metadata;
- graph descriptor -> codec for reloption compatibility checks.

Existing tuple encoding, scan scoring, insert, vacuum, and build flush behavior
remain unchanged.

## Task 63 Handoff

The next RaBitQ slice should extend this adapter rather than adding new
top-level storage-format matches in build/graph first. The expected Task 63
extension points are:

- add a RaBitQ reloption value without changing the HNSW default;
- add a RaBitQ metadata discriminator and codec mapping;
- add a RaBitQ graph storage descriptor/layout only where tuple reads need it;
- route RaBitQ build/insert payload encoding through shared
  `RaBitQQuantizer`;
- prepare scan scoring with shared RaBitQ query state and keep HNSW score
  polarity local to the adapter.

## Validation

- `cargo check -q --lib` passed.
- Plain `cargo test --lib ...` is not a valid local lane for this pgrx crate in
  this session; the test binaries linked but failed at runtime with missing
  PostgreSQL symbols (`CacheRegisterRelcacheCallback` / `LockBuffer`).

See `artifacts/manifest.md` for command metadata.
