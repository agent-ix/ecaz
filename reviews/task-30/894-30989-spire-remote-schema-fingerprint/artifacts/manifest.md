# Review 30989 Artifact Manifest

- head SHA: `76db6ccd1cb351a874719a5cc2ef885a17d03082`
- packet/topic: `30989-spire-remote-schema-fingerprint`
- timestamp: `2026-05-13T16:45:08Z`
- storage format: SPIRE `rabitq` where PG fixtures build SPIRE indexes
- rerank mode: not applicable
- surface: isolated loopback PG18 fixture; no shared-table benchmark surface

## Artifacts

### `cargo-pgrx-test-remote-schema-fingerprint-pg18.log`

- command: `cargo pgrx test pg18 test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql`
- lane / fixture: PG18 focused fixture for remote-only heap type drift
- result: `test tests::pg_test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql ... ok`
- key lines: `1 passed; 0 failed; 1711 filtered out`

### `cargo-pgrx-test-descriptor-contract-pg18.log`

- command: `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_contract`
- lane / fixture: PG18 descriptor contract snapshot
- result: `test tests::pg_test_ec_spire_remote_node_descriptor_contract ... ok`
- key lines: `1 passed; 0 failed; 1711 filtered out`

### `cargo-pgrx-test-registration-contract-pg18.log`

- command: `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_registration_contract`
- lane / fixture: PG18 descriptor registration contract snapshot
- result: `test tests::pg_test_ec_spire_remote_node_descriptor_registration_contract ... ok`
- key lines: `1 passed; 0 failed; 1711 filtered out`

### `cargo-fmt-check.log`

- command: `cargo fmt --check`
- lane / fixture: Rust formatting check
- result: command exited successfully
- key lines: rustfmt emitted only existing stable-toolchain warnings for unstable rustfmt options

### `git-diff-check.log`

- command: `git diff --check`
- lane / fixture: whitespace check
- result: command exited successfully with no output
