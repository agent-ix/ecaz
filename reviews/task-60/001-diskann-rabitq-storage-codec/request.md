# Review Request: DiskANN RaBitQ Storage Codec

## Summary

This checkpoint adds the first `ec_diskann WITH (storage_format = 'rabitq')`
implementation slice.

- Adds `StorageFormat::RaBitQ` parsing while keeping `pq_fastscan` as the default.
- Adds `VAMANA_SEARCH_CODEC_RABITQ` as the metadata discriminator.
- Introduces a DiskANN-local quantizer adapter in `src/am/ec_diskann/quantizer.rs`.
- Uses 1-bit RaBitQ payloads for DiskANN (`DISKANN_RABITQ_BITS = 1`) to match the Task 60 storage goal.
- Reuses the existing Vamana node tuple payload slot: `search_code` stores grouped-PQ bytes for `pq_fastscan` and RaBitQ bytes for `rabitq`.
- Keeps heap exact rerank unchanged.
- Keeps scan-time relation reads on the `RelationGraphReader` path for RaBitQ; grouped-PQ materialization fallback is now limited to grouped-PQ cases.
- Threads RaBitQ through build, empty-index bootstrap, insert payload derivation, and unique-insert planning.

## Commits

- `6a5ed394e46d9ba6fc0e54c69d1789ed5df9840b` Add DiskANN RaBitQ storage codec

## Validation

Packet-local artifacts are in `artifacts/manifest.md`.

- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`

Direct `cargo test --no-default-features --features pg18 rabitq` is not valid in this local shell: the generated pgrx test binary fails at startup with missing PostgreSQL symbol `CacheRegisterRelcacheCallback`.

## Reviewer Focus

- Verify the metadata discriminator is sufficient for backward-compatible `pq_fastscan` reads.
- Check that RaBitQ scan setup does not regress to materializing the full graph chain.
- Check insert/bootstrap parity for RaBitQ, especially the no-codebook path and fixed tuple payload length.
- Check whether `search_subvector_dim` as the RaBitQ bit-width carrier is acceptable for this DiskANN v3 metadata shape.

## Known Remaining Task 60 Work

- Add a benchmark suite packet for 100k and 1M `pq_fastscan` vs `rabitq`.
- Run recall/latency/storage gates and document the acceptable recall delta.
- Prove the 1M size reduction target before calling Task 60 complete.
