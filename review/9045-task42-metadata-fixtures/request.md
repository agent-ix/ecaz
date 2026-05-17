# Review Request: Task 42 metadata fixtures

## Summary

Task 42 follow-up for golden on-disk fixture coverage.

This slice adds the first `fixtures/on-disk/` lane:

- `fixtures/on-disk/hnsw_metadata_v3.hex`;
- `fixtures/on-disk/diskann_vamana_metadata_v3.hex`;
- `tests/on_disk_fixtures.rs`;
- `make on-disk-fixtures`.

The test decodes both metadata fixtures into the expected in-memory values and
then byte-swaps each fixture's format-version field to assert the decoder
rejects the mutated bytes rather than silently interpreting them.

Code commit: `396bae9afacbc5afa84a89fc972667a66f2dca50`

## Review Focus

- Confirm the fixture bytes are stable, hand-pinned little-endian metadata
  payloads rather than bytes regenerated during the test.
- Confirm the swapped-version tests exercise the endian failure mode Task 42
  calls out.
- Confirm `make on-disk-fixtures` is narrow enough for the current pure-codec
  fixture coverage and does not claim full AM/page-kind fixture completion.

## Validation

See `artifacts/manifest.md`.

- `make on-disk-fixtures`
  - Result: 4 passed, 0 failed.
- `make layout-check`
  - Result: 13 passed, 0 failed.
- Note: both runs emitted the existing unused-import warning in `src/am/mod.rs`.

## Remaining Task 42 Gaps

- Extend fixtures beyond HNSW/DiskANN metadata to IVF, SPIRE, codebook payloads,
  and remaining page kinds.
- Extend byte-swapped rejection tests beyond HNSW/DiskANN metadata version
  fields.
- Additional SPIRE routing/top-graph body-prefix assertions if those become
  durable page-buffer contracts beyond the current partition-object codecs.
- qemu cross-arch decode lane with Task 48.
- `(format_version, AM, can_read, can_write)` upgrade matrix.
- WAL record version tags with Task 37.
- pg_upgrade smoke with ECAZ data present.
