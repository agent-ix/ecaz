# Artifact Manifest: 30735 SPIRE Production Receive State Adapter

Head SHA: `dd8c7ab0f97afbf2a3c697eb29308a71ed27b409`
Packet: `review/30735-spire-production-receive-state-adapter`
Lane: Phase 11 Stage C production compact candidate receive
Fixture: local Rust executor-state tests plus matching PG18 dry-state pgrx test
Storage format: state-machine/adapter boundary only; no new index storage
Rerank mode: compact candidate receive pre-heap handoff
Surface isolation: production executor state; AM scan integration still open
Timestamp: 2026-05-10 03:57-03:58 America/Los_Angeles

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
  `production_executor_compact_receive_run_applies_adapter_failure ... ok`
- Key result:
  `test tests::pg_test_ec_spire_production_executor_state_summary_is_dry ... ok`
- Key result:
  `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 1529 filtered out; finished in 13.68s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `git-diff-check.log`

- Command: `git diff --check HEAD~1..HEAD`
- Key result: no whitespace diagnostics for the committed slice
- Exit: `COMMAND_EXIT_CODE="0"`
