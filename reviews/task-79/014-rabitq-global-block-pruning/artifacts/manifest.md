# Task 79 Packet 014 Artifact Manifest

- Head SHA: `fc2b6ca022ba9e6384807ea2c791c6a784b4a034`
- Task bucket: `reviews/task-79/014-rabitq-global-block-pruning/`
- Timestamp: `2026-06-01T17:43:09-07:00`
- Lane: RaBitQ primary
- Storage format: existing V3 leaf summaries; no format bump
- Rerank mode: unchanged by this implementation packet
- Shared-table / isolated table surface: not applicable; code validation only

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Result: passed
- Key lines: command exited 0; only rustfmt stable-channel warnings about ignored unstable import options.

### `cargo-test-global-selector.log`

- Command: `cargo test -p ecaz select_global_leaf_block_row_ranges_spends_budget_across_leaves`
- Result: passed
- Key lines: `test am::ec_spire::scan::tests::select_global_leaf_block_row_ranges_spends_budget_across_leaves ... ok`; `1 passed; 0 failed`.

### `cargo-test-quantized-scan.log`

- Command: `cargo test -p ecaz collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer`
- Result: passed
- Key lines: `test am::ec_spire::scan::tests::collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer ... ok`; `1 passed; 0 failed`.
