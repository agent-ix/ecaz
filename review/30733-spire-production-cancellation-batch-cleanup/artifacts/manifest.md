# Artifact Manifest: 30733 SPIRE Production Cancellation Batch Cleanup

Head SHA: `0c7ab2cf01b9886c598ca7843ea8607b0144db23`
Packet: `review/30733-spire-production-cancellation-batch-cleanup`
Lane: Phase 11 Stage C/C2 production executor cancellation contract
Fixture: local Rust executor-state tests plus matching PG18 dry-state pgrx test
Storage format: state-machine only; no new index storage format
Rerank mode: compact candidate merge pre-heap handoff
Surface isolation: production executor state; no diagnostic live libpq claim
Timestamp: 2026-05-10 03:47-03:48 America/Los_Angeles

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
  `production_executor_local_cancel_clears_ready_candidate_batches ... ok`
- Key result:
  `production_executor_compact_merge_rejects_every_non_ready_state ... ok`
- Key result:
  `test tests::pg_test_ec_spire_production_executor_state_summary_is_dry ... ok`
- Key result:
  `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1529 filtered out; finished in 14.29s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `git-diff-check.log`

- Command: `git diff --check HEAD~1..HEAD`
- Key result: no whitespace diagnostics for the committed slice
- Exit: `COMMAND_EXIT_CODE="0"`
