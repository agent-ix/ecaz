# 30725 Artifacts

Head SHA: `5ee88b3bc170b700ce051610e21a631efc3b0dc6`

Packet/topic: `30725-spire-production-transport-failure-isolation`

Timestamp: `2026-05-10T02:27:20-07:00`

Lane / fixture / storage format / rerank mode:

- Lane: Phase 11 Stage C C1 transport-adapter hardening.
- Fixture: PG18 `pg_test` loopback connection plus a missing local socket
  conninfo for one failed remote.
- Storage format: N/A, no index fixture.
- Rerank mode: N/A, no vector ranking fixture.
- Surface isolation: one local PG18 pg_test instance; this is a transport
  failure-isolation proof, not an isolated one-index-per-table or shared-table
  measurement.

Artifacts:

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Key result: `COMMAND_EXIT_CODE="0"`
- `cargo-check-pg18.log`
  - Command: `cargo check --no-default-features --features pg18`
  - Key result: `Finished dev profile ... target(s) in 0.18s`;
    `COMMAND_EXIT_CODE="0"`
- `cargo-pgrx-pg18-transport-probe.log`
  - Command: `cargo pgrx test pg18 production_transport_probe`
  - Key result:
    `test tests::pg_test_ec_spire_production_transport_probe_isolates_node_failure ... ok`;
    `test tests::pg_test_ec_spire_production_transport_probe_overlaps_ready_remotes ... ok`;
    `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1528 filtered out`;
    `COMMAND_EXIT_CODE="0"`
- `git-diff-check.log`
  - Command:
    `git diff a62c82f383657fe0f1760dea8e1731ab51687cd7 5ee88b3bc170b700ce051610e21a631efc3b0dc6 --check`
  - Key result: `COMMAND_EXIT_CODE="0"`
