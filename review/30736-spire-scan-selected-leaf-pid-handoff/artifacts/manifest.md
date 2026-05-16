# Artifact Manifest: 30736 SPIRE Scan Selected Leaf PID Handoff

Head SHA: `1ff04e6db124e44e436373fce808d552f6c88d6c`
Packet: `review/30736-spire-scan-selected-leaf-pid-handoff`
Lane: Phase 11 Stage C/C5 AM scan remote-fanout precursor
Fixture: local Rust routing-only unit test with one selected remote leaf placement
Storage format: published SPIRE snapshot metadata; no new index storage
Rerank mode: route selection only; no candidate rerank or heap handoff
Surface isolation: routing-only selected PID handoff; existing local routed-row
path remains unchanged
Timestamp: 2026-05-10T11:09:24Z

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

### `cargo-test-selected-leaf-pids.log`

- Command: `cargo test collect_scan_plan_selected_leaf_pids --lib`
- Key result:
  `collect_scan_plan_selected_leaf_pids_does_not_read_remote_leaf_payloads ... ok`
- Key result:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1545 filtered out; finished in 0.00s`
- Exit: `COMMAND_EXIT_CODE="0"`

### `git-diff-check.log`

- Command: `git diff --check HEAD~1..HEAD`
- Key result: no whitespace diagnostics for the committed slice
- Exit: `COMMAND_EXIT_CODE="0"`
