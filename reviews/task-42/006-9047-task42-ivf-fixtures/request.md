# Review Request: Task 42 IVF fixtures

## Summary

Task 42 follow-up for golden on-disk fixture coverage.

This slice extends `fixtures/on-disk/` to the current IVF v1 payloads:

- `fixtures/on-disk/ivf_metadata_v1.hex`;
- `fixtures/on-disk/ivf_centroid_tuple_v1.hex`;
- `fixtures/on-disk/ivf_list_directory_tuple_v1.hex`;
- `fixtures/on-disk/ivf_posting_tuple_v1.hex`;
- `fixtures/on-disk/ivf_pq_codebook_tuple_v1.hex`.

The fixture test decodes each hand-pinned little-endian IVF fixture into the
expected metadata/tuple values. It also byte-swaps the IVF metadata
`format_version` and centroid tuple `dimensions` fields and asserts the decoder
rejects those mutated fixtures.

Code commit: `0852a7d440defe89c1113138aa7de60e3fd3cb58`

## Review Focus

- Confirm the IVF fixture bytes match the documented v1 codec offsets and
  little-endian convention.
- Confirm the IVF codec exports remain limited to the private AM surface and
  `bench_api` for integration tests/benchmarks.
- Confirm the swapped-version and swapped-dimension tests cover bounded IVF
  endian rejection without claiming all fields reject when swapped.
- Confirm `docs/on-disk-format.md` still presents Task 42 as incomplete.

## Validation

See `artifacts/manifest.md`.

- `make on-disk-fixtures`
  - Result: 17 passed, 0 failed.
- `make layout-check`
  - Result: 13 passed, 0 failed.
- Note: both runs emitted the existing unused-import warning in `src/am/mod.rs`.

## Remaining Task 42 Gaps

- Add fixtures for SPIRE and remaining HNSW/DiskANN/IVF page kinds.
- Extend byte-swapped fixture rejection tests to additional bounded multi-byte
  fields where current decoders can reject malformed values.
- Additional SPIRE routing/top-graph body-prefix assertions if those become
  durable page-buffer contracts beyond the current partition-object codecs.
- qemu cross-arch decode lane with Task 48.
- `(format_version, AM, can_read, can_write)` upgrade matrix.
- WAL record version tags with Task 37.
- pg_upgrade smoke with ECAZ data present.
