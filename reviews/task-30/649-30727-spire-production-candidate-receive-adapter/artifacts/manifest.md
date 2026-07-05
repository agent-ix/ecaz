# 30727 Artifacts

Head SHA: `25d8f0e59eeeeae2a56c2c0483a180ba4901c5cc`

Packet/topic: `30727-spire-production-candidate-receive-adapter`

Timestamp: `2026-05-10T02:53:20-07:00`

Lane / fixture / storage format / rerank mode:

- Lane: Phase 11 Stage C C1 async compact-candidate receive adapter.
- Fixture: PG18 loopback connection creates a remote `ec_spire` index and calls
  `ec_spire_remote_search(...)` through `tokio-postgres`.
- Storage format: `rabitq`.
- Rerank mode: N/A, no recall/rerank benchmark.
- Surface isolation: one local PG18 pg_test instance; the receive adapter uses a
  loopback connection and a single remote index fixture.

Artifacts:

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Key result: `COMMAND_EXIT_CODE="0"`
- `cargo-check-pg18.log`
  - Command: `cargo check --no-default-features --features pg18`
  - Key result: `Finished dev profile ... target(s) in 4.76s`;
    `COMMAND_EXIT_CODE="0"`
- `cargo-pgrx-pg18-candidate-receive.log`
  - Command:
    `cargo pgrx test pg18 test_ec_spire_production_candidate_receive_loopback`
  - Key result:
    `test tests::pg_test_ec_spire_production_candidate_receive_loopback ... ok`;
    `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1533 filtered out`;
    `COMMAND_EXIT_CODE="0"`
- `git-diff-check.log`
  - Command:
    `git diff 8a9f8781e5f14379511d7d803e8c6d7ece406deb 25d8f0e59eeeeae2a56c2c0483a180ba4901c5cc --check`
  - Key result: `COMMAND_EXIT_CODE="0"`
