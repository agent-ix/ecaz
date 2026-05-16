# SPIRE Manifest Persist Epoch Guard Artifact Manifest

Head SHA: `9bb3d383`
Packet/topic: `30632-spire-manifest-persist-epoch-guard`

This packet makes no measurement claim.

| Artifact | Lane | Fixture | Storage format | Rerank mode | Command | Timestamp | Isolated one-index-per-table | Key result lines |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| none | manifest persistence validation | PG18 focused manifest persistence tests | n/a | n/a | `cargo test --no-default-features --features "pg18 pg_test" remote_epoch_manifest_persist` | 2026-05-08 | n/a | `test tests::pg_test_ec_spire_remote_epoch_manifest_persist_ready ... ok`; `test tests::pg_test_ec_spire_remote_epoch_manifest_persist_blocked - should panic ... ok` |

## Files

- `src/am/ec_spire/root/snapshots.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
