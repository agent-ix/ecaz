# 30726 Artifacts

Head SHA: `394582e18d194d9a757e7d8064c2acccf83d6a2a`

Packet/topic: `30726-spire-production-transport-state`

Timestamp: `2026-05-10T02:40:54-07:00`

Lane / fixture / storage format / rerank mode:

- Lane: Phase 11 Stage C C1 production executor transport-state integration.
- Fixture: Rust production executor state rows plus PG18 dry production-state
  summary.
- Storage format: N/A for state unit tests; PG18 dry summary uses local
  `ecvector_spire_ip_ops` test index with remote descriptor metadata only.
- Rerank mode: N/A, no recall/rerank benchmark.
- Surface isolation: one local PG18 pg_test instance; no remote socket is opened
  by the dry SQL summary.

Artifacts:

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Key result: `COMMAND_EXIT_CODE="0"`
- `cargo-check-pg18.log`
  - Command: `cargo check --no-default-features --features pg18`
  - Key result: `Finished dev profile ... target(s) in 0.20s`;
    `COMMAND_EXIT_CODE="0"`
- `cargo-test-production-executor-state.log`
  - Command: `cargo test --no-default-features --features pg18 production_executor_state_`
  - Key result:
    `production_executor_state_moves_ready_transport_to_candidate_receive ... ok`;
    `production_executor_state_preserves_transport_failure_category ... ok`;
    `production_executor_state_rejects_unplanned_transport_result ... ok`;
    `pg_test_ec_spire_production_executor_state_summary_is_dry ... ok`;
    `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1527 filtered out`;
    `COMMAND_EXIT_CODE="0"`
- `git-diff-check.log`
  - Command:
    `git diff ca0faede68423565cea7204d391f77a0f29599cc 394582e18d194d9a757e7d8064c2acccf83d6a2a --check`
  - Key result: `COMMAND_EXIT_CODE="0"`
