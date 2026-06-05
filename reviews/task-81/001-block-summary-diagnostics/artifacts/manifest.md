# Task 81 Packet 001 Artifact Manifest

- Head SHA before slice: `2894ef0d147099054475a217d5feb8984219b789`
- Task bucket: `reviews/task-81/001-block-summary-diagnostics`
- Timestamp: `2026-06-04T18:11:33-07:00`
- Lane / fixture / storage format / rerank mode: code validation only; no benchmark lane in this packet.
- Storage surface: existing isolated leaf object surfaces; V2 row-payload fallback, V3 single-representative block summaries, V4 multi-representative block summaries.

## Artifacts

### `cargo-check-pg18.log`

- Command: `cargo check --no-default-features --features pg18`
- Result: passed.
- Key line: `Finished dev profile [unoptimized + debuginfo]`

### `cargo-test-leaf-partition-v.log`

- Command: `cargo test --no-default-features --features pg18 leaf_partition_object_v`
- Result: passed.
- Key line: `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1962 filtered out`
- Also ran matching on-disk fixture filters: `2 passed; 0 failed`.

### `cargo-test-global-block-selection.log`

- Command: `cargo test --no-default-features --features pg18 select_global_leaf_block_row_ranges_uses_rabitq_summary_radius`
- Result: passed.
- Key line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1969 filtered out`

### `cargo-test-scan-diagnostics.log`

- Command: `cargo test --no-default-features --features pg18 collect_scan_placement_diagnostics`
- Result: passed.
- Key line: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1966 filtered out`

## Notes

- `cargo fmt --check` was attempted before packet logging and failed on an unrelated pre-existing diff in `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`; this packet did not modify that file.
