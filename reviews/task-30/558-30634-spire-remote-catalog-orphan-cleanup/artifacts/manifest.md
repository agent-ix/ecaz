# SPIRE Remote Catalog Orphan Cleanup Artifact Manifest

Head SHA: `b06aab0b`
Packet/topic: `30634-spire-remote-catalog-orphan-cleanup`

This packet makes no measurement claim.

| Artifact | Lane | Fixture | Storage format | Rerank mode | Command | Timestamp | Isolated one-index-per-table | Key result lines |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| none | lifecycle cleanup validation | PG18 synthetic dead coordinator OID | n/a | n/a | `cargo pgrx test pg18 test_ec_spire_remote_catalog_orphan_cleanup` | 2026-05-08 | n/a | `test tests::pg_test_ec_spire_remote_catalog_orphan_cleanup ... ok` |

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
