# Review Request: HNSW RaBitQ V4 Format Fixture

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- packet: `reviews/task-63/013-hnsw-rabitq-v4-format-fixture/`

## Summary

This packet closes a local Task 63 on-disk-format documentation gap. HNSW
RaBitQ introduced metadata format tag `4`, but the Task 42 format docs and
upgrade matrix still listed HNSW tags only through `3`.

## Touched Files

- `fixtures/on-disk/hnsw_metadata_v4_rabitq.hex`
  - adds a minimal HNSW V4 RaBitQ metadata fixture.
- `tests/on_disk_fixtures.rs`
  - decodes the V4 fixture;
  - verifies the RaBitQ-specific metadata fields;
  - verifies a byte-swapped V4 format version is rejected.
- `fixtures/upgrade/matrix.csv` and `tests/upgrade_matrix.rs`
  - register HNSW V4 as readable/writable alongside HNSW V3, because
    `storage_format` selects the writable HNSW format.
- `docs/on-disk-format.md`
  - documents HNSW tag `4`, the new fixture, and the two-writable-HNSW-format
    state.

## Validation

Packet-local logs:

- `artifacts/cargo-test-on-disk-hnsw-v4-rabitq.log`
  - `cargo test -q --test on_disk_fixtures hnsw_metadata_v4_rabitq`
  - passed: `2 passed; 0 failed`.
- `artifacts/cargo-test-upgrade-matrix.log`
  - `cargo test -q --test upgrade_matrix`
  - passed: `2 passed; 0 failed`.

No benchmarks were run.
