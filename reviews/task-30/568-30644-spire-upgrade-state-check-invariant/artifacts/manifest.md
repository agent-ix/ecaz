# Artifact Manifest

Packet: `30644-spire-upgrade-state-check-invariant`

This packet makes no measurement claim and has no raw benchmark artifacts.

| Artifact | Head SHA | Lane / fixture | Command | Timestamp | Key result lines |
| --- | --- | --- | --- | --- | --- |
| none | `159556a9` | PG18 bootstrap/upgrade descriptor-state CHECK invariant | `cargo pgrx test pg18 test_ec_spire_remote_state_upgrade_check_matches_bootstrap` | 2026-05-09T04:13:09Z | `test tests::pg_test_ec_spire_remote_state_upgrade_check_matches_bootstrap ... ok`; `test result: ok. 1 passed; 0 failed` |
