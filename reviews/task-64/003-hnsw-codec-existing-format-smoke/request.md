# Review Request: HNSW Codec Existing-Format Smoke

## Summary

This packet closes the Task 64 validation loop for the existing HNSW formats
after the Task 63 RaBitQ integration. It is an audit/evidence packet against
head `8c8577dae8eda13741031f91ab5fbb65f41dfca9`; no new code is introduced in
this packet.

Task 64 introduced the HNSW-local codec identity adapter in earlier packets and
Task 63 then consumed that shape for RaBitQ. This packet checks that the two
pre-existing formats still work through build, scan, insert, delete, and
vacuum:

- `storage_format = 'turboquant'`
- `storage_format = 'pq_fastscan'`

## Validation

Packet-local evidence is recorded in `artifacts/manifest.md`.

- `ecaz-dev-sql-pg18-hnsw-existing-formats-setup.log`: creates matched
  TurboQuant and PqFastScan HNSW indexes, forces indexed scans, verifies the
  build-time nearest row, inserts a live row into each table, and verifies the
  live row ranks first for its own query.
- `ecaz-dev-sql-pg18-hnsw-existing-formats-delete.log`: deletes the original
  nearest row from both tables and verifies row counts.
- `ecaz-dev-sql-pg18-hnsw-existing-formats-vacuum-turboquant.log`: standalone
  `VACUUM` for the TurboQuant table.
- `ecaz-dev-sql-pg18-hnsw-existing-formats-vacuum-pq.log`: standalone
  `VACUUM` for the PqFastScan table.
- `ecaz-dev-sql-pg18-hnsw-existing-formats-post-vacuum.log`: forces
  post-vacuum indexed scans for both formats and asserts deleted id `1` is not
  returned.

Key passing result lines:

- `task64_hnsw_existing_formats_setup_insert_passed |                  16384 |          16384`
- `task64_hnsw_existing_formats_delete_passed |                         6 |                 6`
- `VACUUM`
- `task64_hnsw_existing_formats_post_vacuum_passed |                  16384 |          16384`

## Adapter Audit

The current adapter shape remains HNSW-local:

- `src/am/ec_hnsw/codec.rs` owns storage-format identity, metadata identity,
  reloption names, and build tuple sizing.
- `src/am/ec_hnsw/graph.rs` owns metadata-to-layout validation and keeps
  tuple-body shape local to HNSW graph storage.
- `src/am/ec_hnsw/insert.rs` and `src/am/ec_hnsw/vacuum.rs` preserve shared
  graph topology logic while dispatching payload encoding/retention by
  descriptor.
- `src/am/ec_hnsw/scan.rs` keeps traversal scoring prepared state local to HNSW
  scan execution rather than introducing a cross-AM trait.

This is consistent with ADR-071/ADR-072: shared quantizer math remains in
`src/quant`, while the AM-specific codec binding stays local until DiskANN and
HNSW prove a stable repeated shape.
