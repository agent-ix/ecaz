# Review Request: Task 42 SPIRE metadata fixtures

## Summary

Task 42 follow-up for golden on-disk fixture coverage.

This slice adds hand-pinned SPIRE metadata-format v1 fixtures:

- `fixtures/on-disk/spire_local_store_config_v1.hex`;
- `fixtures/on-disk/spire_placement_entry_v1.hex`;
- `fixtures/on-disk/spire_placement_directory_v1.hex`;
- `fixtures/on-disk/spire_epoch_manifest_v1.hex`;
- `fixtures/on-disk/spire_manifest_entry_v1.hex`;
- `fixtures/on-disk/spire_object_manifest_v1.hex`.

The fixture test decodes each fixture into expected values and byte-swaps each
fixture's metadata format-version field to assert the decoder rejects the
mutated bytes.

Code commit: `82bd0565fe9a098bf6ce70f8c9ed598c22aa86b8`

## Review Focus

- Confirm the SPIRE fixture bytes match the existing metadata offset constants
  and little-endian convention.
- Confirm the SPIRE metadata decode exports are limited to the private AM
  surface plus `bench_api`.
- Confirm the swapped-version tests exercise rejection without claiming full
  byte-swapped coverage for every multi-byte metadata field.
- Confirm `docs/on-disk-format.md` still records remaining Task 42 gaps.

## Validation

See `artifacts/manifest.md`.

- `make on-disk-fixtures`
  - Result: 29 passed, 0 failed.
- `make layout-check`
  - Result: 13 passed, 0 failed.
- Note: both runs emitted the existing unused-import warning in `src/am/mod.rs`.

## Remaining Task 42 Gaps

- Add fixtures for SPIRE partition object bodies and remaining HNSW/DiskANN/IVF
  page kinds.
- Extend byte-swapped fixture rejection tests to additional bounded multi-byte
  fields where current decoders can reject malformed values.
- Additional SPIRE routing/top-graph body-prefix assertions if those become
  durable page-buffer contracts beyond the current partition-object codecs.
- qemu cross-arch decode lane with Task 48.
- `(format_version, AM, can_read, can_write)` upgrade matrix.
- WAL record version tags with Task 37.
- pg_upgrade smoke with ECAZ data present.
