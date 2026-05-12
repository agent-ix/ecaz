# Artifact Manifest: 30936 SPIRE UPDATE/DELETE Schema Drift

Head SHA: `ebbc9dba7578b42db4b24ae8663b6938f738fe90`

Packet/topic: `30936-spire-update-delete-schema-drift`

Timestamp: `2026-05-12T22:17:11Z`

This packet does not make throughput or capacity measurement claims.

## Artifacts

- `git-diff-check.log`
  - Command: `git diff --check HEAD^ HEAD`
  - Lane: static whitespace validation
  - Key result: command exited successfully.

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Lane: Rust formatting validation
  - Key result: command exited successfully. The log includes rustfmt warnings
    about unstable import grouping options already present in the repo config.

- `cargo-pgrx-test-update-delete-schema-drift.log`
  - Command:
    `cargo pgrx test pg18 test_ec_spire_update_delete_schema_drift_guard_sql`
  - Lane: PG18 focused coordinator-routed UPDATE/DELETE schema-drift fixture
  - Fixture: loopback one-coordinator/one-remote table pair
  - Rerank mode: not applicable
  - Surface: shared `ec_spire_remote_node_descriptor` and shared
    `ec_spire_placement`
  - Key result: `1 passed; 0 failed; 1687 filtered out`.
