# SPIRE Coordinator Pipeline Bundle Artifact Manifest

Head SHA: `6ab8c8b3`
Packet/topic: `30631-spire-coordinator-pipeline-bundle`

This packet makes no measurement claim.

| Artifact | Lane | Fixture | Storage format | Rerank mode | Command | Timestamp | Isolated one-index-per-table | Key result lines |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| none | coordinator gate validation | PG18 focused coordinator-gate test | n/a | n/a | `cargo test --no-default-features --features "pg18 pg_test" test_ec_spire_remote_search_coordinator_gate_summary` | 2026-05-08 | n/a | `test tests::pg_test_ec_spire_remote_search_coordinator_gate_summary ... ok` |

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
