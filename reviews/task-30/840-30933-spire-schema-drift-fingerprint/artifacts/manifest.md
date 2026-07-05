# Artifact Manifest: 30933 SPIRE Schema Drift Fingerprint

Head SHA: `369c50d1c57641f6f7b8a9a8bd4656623d3ffdbd`

Packet/topic: `30933-spire-schema-drift-fingerprint`

Timestamp: `2026-05-12T22:01:33Z`

Storage surface: coordinator-routed INSERT descriptor catalog and loopback
remote fixture. This packet does not make throughput or capacity measurement
claims.

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

- `cargo-pgrx-test-schema-drift.log`
  - Command:
    `cargo pgrx test pg18 test_ec_spire_schema_drift_fails_before_dispatch_sql`
  - Lane: PG18 focused coordinator-routed INSERT schema-drift fixture
  - Fixture: loopback one-coordinator/one-remote table pair
  - Rerank mode: not applicable
  - Surface: shared `ec_spire_remote_node_descriptor` and shared
    `ec_spire_placement`
  - Key result: `1 passed; 0 failed; 1686 filtered out`.

- `cargo-pgrx-test-descriptor-contract.log`
  - Command:
    `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_contract`
  - Lane: PG18 descriptor contract fixture
  - Fixture: descriptor contract SQL surface
  - Rerank mode: not applicable
  - Surface: shared `ec_spire_remote_node_descriptor`
  - Key result: `1 passed; 0 failed; 1686 filtered out`.
