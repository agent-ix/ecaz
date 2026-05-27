# Artifact Manifest: Task 63 HNSW RaBitQ V4 Format Fixture

- head SHA: `133cd263a1a7b53d56b0e2146eca37aeeeecb0e3`
- task bucket: `reviews/task-63/`
- packet path: `reviews/task-63/013-hnsw-rabitq-v4-format-fixture/`
- lane: HNSW RaBitQ on-disk format fixture and upgrade matrix
- timestamp: 2026-05-27 America/Los_Angeles

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-test-on-disk-hnsw-v4-rabitq.log` | `cargo test -q --test on_disk_fixtures hnsw_metadata_v4_rabitq` | passed, 2/2 |
| `cargo-test-upgrade-matrix.log` | `cargo test -q --test upgrade_matrix` | passed, 2/2 |

## Notes

This packet records fixture and matrix coverage only. It does not run any HNSW
benchmark matrix.
