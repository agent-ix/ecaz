# Artifact Manifest

Packet: `30774-spire-remote-manifest-freshness-diagnostics`
Head SHA: `0b66c8a2a8dcdd97fc8de618bd4de509a54daa12`
Timestamp: `2026-05-10T22:01:02Z`

## Validation Artifacts

- `cargo fmt --check`
  - Head SHA: `0b66c8a2a8dcdd97fc8de618bd4de509a54daa12`
  - Lane / fixture / storage format / rerank mode: formatting check; no
    PostgreSQL fixture; storage/rerank not applicable
  - Command: `cargo fmt --check`
  - Key result: exited 0

- `git diff --check`
  - Head SHA: `0b66c8a2a8dcdd97fc8de618bd4de509a54daa12`
  - Lane / fixture / storage format / rerank mode: whitespace check for touched
    files; no PostgreSQL fixture; storage/rerank not applicable
  - Command: `git diff --check -- src/lib.rs src/am/ec_spire/root/remote_candidates.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`
  - Key result: exited 0

- `cargo check --no-default-features --features pg18`
  - Head SHA: `0b66c8a2a8dcdd97fc8de618bd4de509a54daa12`
  - Lane / fixture / storage format / rerank mode: PG18 compile check; no
    PostgreSQL fixture; storage/rerank not applicable
  - Command: `cargo check --no-default-features --features pg18`
  - Key result: `Finished dev profile`

- `cargo check --no-default-features --features "pg18 pg_test"`
  - Head SHA: `0b66c8a2a8dcdd97fc8de618bd4de509a54daa12`
  - Lane / fixture / storage format / rerank mode: PG18 pg_test compile check;
    no PostgreSQL fixture; storage/rerank not applicable
  - Command: `cargo check --no-default-features --features "pg18 pg_test"`
  - Key result: `Finished dev profile`

- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`
  - Head SHA: `0b66c8a2a8dcdd97fc8de618bd4de509a54daa12`
  - Lane / fixture / storage format / rerank mode: PG18 pgrx single-instance
    manifest fixture; ec_spire index; RaBitQ scoring path not exercised
  - Command: `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`
  - Isolated one-index-per-table or shared-table surface:
    isolated test table/index
  - Key result: `test tests::pg_test_ec_spire_remote_epoch_manifest_persist_ready ... ok`

- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
  - Head SHA: `0b66c8a2a8dcdd97fc8de618bd4de509a54daa12`
  - Lane / fixture / storage format / rerank mode: PG18 pgrx contract fixture;
    SQL catalog/operator contract; storage/rerank not applicable
  - Command: `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
  - Isolated one-index-per-table or shared-table surface:
    contract-only SQL surface
  - Key result: `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`
