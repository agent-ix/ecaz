---
topic: spire-delete-not-found-idempotence
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30920
---

# Artifact Manifest

Head SHA: `006167504ecd10b17104f862cc346d94e1211ffd`

Packet/topic: `30920-spire-delete-not-found-idempotence`

Timestamp: `2026-05-12T12:30:42-07:00`

Surface: local PG18 pgrx coordinator DELETE helper tests.

## Artifacts

### `git-diff-check.log`

- Command: `git diff --check HEAD^ HEAD`
- Exit code: 0
- Key result: no whitespace errors.

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Exit code: 0
- Key result: formatting check passed. The log contains the existing stable
  toolchain warnings for unstable rustfmt import-group options.

### `cargo-pgrx-test-delete-idempotence.log`

- Command: `cargo pgrx test pg18 test_ec_spire_prepare_coordinator_delete`
- Exit code: 0
- Lane / fixture: PG18 coordinator DELETE helper fixtures.
- Storage format / rerank mode: `storage_format = 'rabitq'`, no rerank mode
  override.
- Shared-table vs isolated: isolated test tables and indexes; shared
  `ec_spire_placement` catalog surface.
- Key result lines:
  - `test tests::pg_test_ec_spire_prepare_coordinator_delete_idempotent_sql ... ok`
  - `test tests::pg_test_ec_spire_prepare_coordinator_delete_local_sql ... ok`
  - `test tests::pg_test_ec_spire_prepare_coordinator_delete_tuple_payload_sql ... ok`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1678 filtered out`
