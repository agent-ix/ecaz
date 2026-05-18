# Artifact Manifest

Packet: `30643-spire-secret-key-collision-guard`

This packet makes no measurement claim and has no raw benchmark artifacts.

| Artifact | Head SHA | Lane / fixture | Command | Timestamp | Key result lines |
| --- | --- | --- | --- | --- | --- |
| none | `3ab80414` | PG18 descriptor registration secret-key collision | `cargo pgrx test pg18 test_ec_spire_remote_secret_key_collision_rejected` | 2026-05-09T04:08:04Z | `test tests::pg_test_ec_spire_remote_secret_key_collision_rejected - should panic ... ok`; `test result: ok. 1 passed; 0 failed` |
| none | `3ab80414` | PG18 descriptor field contract | `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_contract` | 2026-05-09T04:08:04Z | `test tests::pg_test_ec_spire_remote_node_descriptor_contract ... ok`; `test result: ok. 1 passed; 0 failed` |
| none | `3ab80414` | PG18 descriptor registration contract | `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_registration_contract` | 2026-05-09T04:08:04Z | `test tests::pg_test_ec_spire_remote_node_descriptor_registration_contract ... ok`; `test result: ok. 1 passed; 0 failed` |
