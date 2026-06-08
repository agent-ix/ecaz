head_sha: a96bf29088c2c834aedc5cd7732a9a60b9096fa1
task_bucket: reviews/task-85
packet_path: reviews/task-85/015-legacy-snapshot-fallback
timestamp: 2026-06-07
lane: local-validation
fixture: none
storage_format: not applicable
rerank_mode: not applicable
surface: CLI compatibility for retained AWS extension signatures
isolated_one_index_per_table: not applicable

# Artifacts

- `cargo-fmt-check.log`
  - command: `cargo fmt --check`
  - result: passed with existing stable-rustfmt warnings.
- `cargo-test-ecaz-cli-spire-pipeline.log`
  - command: `cargo test --manifest-path crates/ecaz-cli/Cargo.toml spire_pipeline --locked --offline`
  - result: passed; 21 tests passed, 0 failed.

# Key Result Lines

- `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 385 filtered out`.

# Implementation Notes

`ecaz bench spire-pipeline` now tries the new
`leaf_row_segment_read_count` / `leaf_row_segment_read_bytes` snapshot columns
first. If the retained database still has the legacy pgrx SQL return
signature, it falls back to the legacy column list and reports row-segment
counters as zero.

This preserves AWS 1M retained-table benchmarkability after
`--skip-extension-recreate`, while still allowing fresh extension installs to
emit actual selected row-segment metrics.
