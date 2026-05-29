# Review Request: HNSW RaBitQ Lifecycle Smoke

## Summary

This packet covers commit `8441806a59957a456cb11429860ad54a17133f51` (`Add HNSW RaBitQ lifecycle coverage`).

The code change adds focused HNSW RaBitQ lifecycle coverage:

- Extends the grouped hot/cold decode helper to accept both `PqFastScan` and `RaBitQ` descriptors.
- Adds `test_ech_rabitq_build_scan_insert_vacuum_round_trip`, which builds a source-backed RaBitQ HNSW index, verifies V4 RaBitQ metadata and grouped hot tuple shape, scans build-time rows, inserts a live row, and validates vacuum removal of a deleted heap TID.

## Validation

Packet-local evidence is recorded in `artifacts/manifest.md`.

- `cargo-test-hnsw-rabitq-lifecycle-no-run.log`: focused pg_test compile validation passed.
- `ecaz-dev-sql-pg18-hnsw-rabitq-lifecycle-setup.log`: PG18 SQL smoke built a RaBitQ HNSW index, forced an index scan, verified nearest build-time row, inserted a live row, and verified the live row ranks first for its own query.
- `ecaz-dev-sql-pg18-hnsw-rabitq-delete.log`: deleted the original nearest row and confirmed eight rows remain.
- `ecaz-dev-sql-pg18-hnsw-rabitq-vacuum.log`: ran table `VACUUM` as a standalone PG18 command.
- `ecaz-dev-sql-pg18-hnsw-rabitq-post-vacuum.log`: forced a post-vacuum index scan and asserted the deleted row is absent.

Key passing SQL result lines:

- `task63_hnsw_rabitq_setup_insert_passed |       16384`
- `task63_hnsw_rabitq_delete_passed |              8`
- `VACUUM`
- `task63_hnsw_rabitq_post_vacuum_passed |       16384`

## Notes

The earlier combined SQL smoke in `ecaz-dev-sql-pg18-hnsw-rabitq-lifecycle.log` is retained as exploratory evidence only. PostgreSQL rejected `VACUUM` inside the implicit transaction created by a multi-statement `psql -c`, so the passing validation splits vacuum into its own command.
