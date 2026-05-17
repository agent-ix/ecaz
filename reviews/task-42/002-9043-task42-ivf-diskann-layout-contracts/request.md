# Review Request: Task 42 IVF and DiskANN layout contracts

## Summary

Task 42 follow-up for static on-disk layout invariants.

This slice extends `tests/size_of_assertions.rs` beyond the prior generic,
HNSW, and DiskANN metadata coverage by pinning encoded byte contracts for:

- DiskANN Vamana node tuple fixed header offsets;
- DiskANN Vamana node dynamic-region offset helpers for binary words, search
  code, and neighbors;
- DiskANN Vamana grouped-PQ codebook tuple offsets;
- IVF metadata payload size, magic, format version, and field offsets;
- IVF block refs, centroid tuples, list-directory tuples, posting tuples, and
  PQ-codebook tuple offsets.

`docs/on-disk-format.md` is updated so the remaining static-layout gap is now
SPIRE storage/meta coverage.

Code commit: `046bcb246a9ccd85587fd00285f9b66018ac1b0d`

## Review Focus

- Confirm the new IVF constants mirror the existing slice indexes in
  `src/am/ec_ivf/page.rs`.
- Confirm the DiskANN node tuple helpers correctly account for variable
  `binary_word_count` and `search_code_len`.
- Confirm the new `bench_api` exports remain limited to layout-check support.

## Validation

See `artifacts/manifest.md`.

- `cargo test --features bench --test size_of_assertions`
  - Result: 13 passed, 0 failed.
  - Note: the run emitted one existing unused-import warning in `src/am/mod.rs`.

## Remaining Task 42 Gaps

- Golden fixtures under `fixtures/on-disk/`.
- Byte-swapped fixture rejection tests.
- Static offset coverage for SPIRE partition objects, leaf V2, chain objects,
  placement metadata, and epoch records.
- qemu cross-arch decode lane with Task 48.
- `(format_version, AM, can_read, can_write)` upgrade matrix.
- WAL record version tags with Task 37.
- pg_upgrade smoke with ECAZ data present.
