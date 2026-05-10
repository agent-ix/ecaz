# Artifact Manifest: 30739 SPIRE Remote Backend Termination Taxonomy

Head SHA: `4f2e826dcb70159c91ea70895f139a2ec5a8e64f`
Packet: `review/30739-spire-remote-backend-termination-taxonomy`
Lane: Phase 11 Stage C2 fault taxonomy
Fixture: PG18 loopback production transport probe that terminates its remote
backend
Storage format: no index storage changes
Rerank mode: not applicable
Surface isolation: async production transport adapter; compact receive and
strict/degraded matrix expansion remain open
Timestamp: 2026-05-10T11:34:55Z

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

### `cargo-pgrx-test-backend-terminated.log`

- Command:
  `cargo pgrx test pg18 test_ec_spire_prod_transport_backend_terminated`
- Key result:
  `test tests::pg_test_ec_spire_prod_transport_backend_terminated ... ok`
- Key result:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1548 filtered out; finished in 23.97s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `git-diff-check.log`

- Command: `git diff --check HEAD~1..HEAD`
- Key result: no whitespace diagnostics for the committed slice
- Exit: `COMMAND_EXIT_CODE="0"`
