# 30724 Artifacts

Head SHA: `33796ac1beae8350c82740f47d07ea4e1d3217ce`

Packet/topic: `30724-spire-production-transport-probe-adapter`

Timestamp: `2026-05-10T02:16:30-07:00`

Lane / fixture / storage format / rerank mode:

- Lane: Phase 11 Stage C C1 transport-adapter proof.
- Fixture: PG18 `pg_test` loopback connection with two probe requests:
  slow `SELECT pg_sleep(0.30)` and fast `SELECT 1`.
- Storage format: N/A, no index fixture.
- Rerank mode: N/A, no vector ranking fixture.
- Surface isolation: one local PG18 pg_test instance; this is a transport
  progress proof, not an isolated one-index-per-table or shared-table
  measurement.

Artifacts:

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Key result: `COMMAND_EXIT_CODE="0"`
- `cargo-check-pg18.log`
  - Command: `cargo check --no-default-features --features pg18`
  - Key result: `Finished dev profile ... target(s) in 0.12s`;
    `COMMAND_EXIT_CODE="0"`
- `cargo-pgrx-pg18-transport-probe.log`
  - Command:
    `cargo pgrx test pg18 test_ec_spire_production_transport_probe_overlaps_ready_remotes`
  - Key result:
    `test tests::pg_test_ec_spire_production_transport_probe_overlaps_ready_remotes ... ok`;
    `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1528 filtered out`;
    `COMMAND_EXIT_CODE="0"`
- `git-diff-check.log`
  - Command:
    `git diff 96c4694f36e73a722e7db6bd4618894a1da1f1a5 33796ac1beae8350c82740f47d07ea4e1d3217ce --check`
  - Key result: `COMMAND_EXIT_CODE="0"`
