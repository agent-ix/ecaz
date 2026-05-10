# Artifact Manifest: 30738 SPIRE Remote Statement Timeout Taxonomy

Head SHA: `3894097a58a25327ad125fa35568a9ec61333a1d`
Packet: `review/30738-spire-remote-statement-timeout-taxonomy`
Lane: Phase 11 Stage C2 failure taxonomy
Fixture: PG18 loopback transport probe and compact-candidate receive timeout
fixtures
Storage format: no index storage changes
Rerank mode: compact candidate receive pre-heap handoff
Surface isolation: async production transport adapter and compact-candidate
receive adapter; full cancellation propagation remains open
Timestamp: 2026-05-10T11:27:03Z

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Key result: `COMMAND_EXIT_CODE="0"`
- Note: existing stable-rustfmt warnings for unstable
  `imports_granularity` / `group_imports` settings are present.

### `cargo-check-pg18.log`

- Command: `cargo check --no-default-features --features pg18`
- Key result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.12s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `cargo-test-production-executor-lib.log`

- Command: `cargo test production_executor_ --lib`
- Key result:
  `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 1532 filtered out; finished in 14.07s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `cargo-pgrx-test-transport-timeout.log`

- Command:
  `cargo pgrx test pg18 test_ec_spire_prod_transport_remote_stmt_timeout`
- Key result:
  `test tests::pg_test_ec_spire_prod_transport_remote_stmt_timeout ... ok`
- Key result:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1547 filtered out; finished in 24.34s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `cargo-pgrx-test-receive-timeout.log`

- Command:
  `cargo pgrx test pg18 test_ec_spire_prod_receive_remote_stmt_timeout`
- Key result:
  `test tests::pg_test_ec_spire_prod_receive_remote_stmt_timeout ... ok`
- Key result:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1547 filtered out; finished in 24.60s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `git-diff-check.log`

- Command: `git diff --check HEAD~1..HEAD`
- Key result: no whitespace diagnostics for the committed slice
- Exit: `COMMAND_EXIT_CODE="0"`
