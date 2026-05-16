# Artifact Manifest: 30740 SPIRE Remote Cancel Fault Taxonomy

Head SHA: `a1c02ce982ca5121e6860790fdfc39e78897258e`
Packet: `review/30740-spire-remote-cancel-fault-taxonomy`
Lane: Phase 11 Stage C2 fault taxonomy
Fixture: PG18 loopback production transport and compact-candidate receive
remote cancel/backend termination fixtures
Storage format: no index storage changes
Rerank mode: compact candidate receive pre-heap handoff
Surface isolation: async production transport and compact-candidate receive
taxonomy only; local cancellation propagation remains open
Timestamp: 2026-05-10T11:48:21Z

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Key result: `COMMAND_EXIT_CODE="0"`
- Note: existing stable-rustfmt warnings for unstable
  `imports_granularity` / `group_imports` settings are present.

### `cargo-check-pg18.log`

- Command: `cargo check --no-default-features --features pg18`
- Key result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.15s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `cargo-pgrx-test-remote-query-cancelled.log`

- Command: `cargo pgrx test pg18 remote_query_cancelled`
- Key result:
  `test tests::pg_test_ec_spire_prod_transport_remote_query_cancelled ... ok`
- Key result:
  `test tests::pg_test_ec_spire_prod_receive_remote_query_cancelled ... ok`
- Key result:
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1550 filtered out; finished in 23.36s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `cargo-pgrx-test-receive-backend-terminated.log`

- Command:
  `cargo pgrx test pg18 test_ec_spire_prod_receive_backend_terminated`
- Key result:
  `test tests::pg_test_ec_spire_prod_receive_backend_terminated ... ok`
- Key result:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1551 filtered out; finished in 23.37s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `git-diff-check.log`

- Command: `git diff --check HEAD~1..HEAD`
- Key result: no whitespace diagnostics for the committed slice
- Exit: `COMMAND_EXIT_CODE="0"`
