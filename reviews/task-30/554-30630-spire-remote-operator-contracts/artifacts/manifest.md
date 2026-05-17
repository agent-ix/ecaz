# SPIRE Remote Operator Contracts Artifact Manifest

Head SHA: `1e17c28d`
Packet/topic: `30630-spire-remote-operator-contracts`

This packet makes no measurement claim.

| Artifact | Lane | Fixture | Storage format | Rerank mode | Command | Timestamp | Isolated one-index-per-table | Key result lines |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| none | contract validation | PG18 focused policy contract test | n/a | n/a | `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts` | 2026-05-08 | n/a | `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok` |

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
