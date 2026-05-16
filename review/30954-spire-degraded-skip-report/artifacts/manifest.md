# Artifact Manifest: SPIRE Degraded Skip Report

- Head SHA: `f3a6c65199aa2f4be21bec7896509db95c437f71`
- Packet/topic: `30954-spire-degraded-skip-report`
- Timestamp: `2026-05-13T01:28:27Z`
- Surface: Phase 12.7 degraded skipped/stale remote node reporting
- Lane / fixture / storage format / rerank mode: Rust unit diagnostic helper;
  no SQL fixture; n/a; n/a.
- Isolation surface: unit diagnostic helper; no isolated or shared-table
  runtime surface.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check f3a6c651^ f3a6c651" review/30954-spire-degraded-skip-report/artifacts/git-diff-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30954-spire-degraded-skip-report/artifacts/cargo-fmt-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`

### `cargo-test-degraded-skip-report.log`

- Command:
  `script -q -c "cargo test degraded_skip_report_lists_each_skipped_node --lib" review/30954-spire-degraded-skip-report/artifacts/cargo-test-degraded-skip-report.log`
- Key result lines:
  - `test am::ec_spire::production_executor_state_tests::degraded_skip_report_lists_each_skipped_node ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1690 filtered out`
  - `COMMAND_EXIT_CODE="0"`
