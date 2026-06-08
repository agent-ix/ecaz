head_sha: 1918e773e73fdbf21c2d16ca497b539ba5276d81
task_bucket: reviews/task-85
packet_path: reviews/task-85/017-append-only-row-segment-snapshot
timestamp: 2026-06-07
lane: local-validation
fixture: none
storage_format: not applicable
rerank_mode: not applicable
surface: append-only SPIRE leaf snapshot row-segment metrics
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

`leaf_row_segment_read_count` and `leaf_row_segment_read_bytes` now append to
`ec_spire_index_scan_leaf_candidate_snapshot` instead of inserting before
`primary_candidate_row_count`. The CLI current-signature query decodes those
fields at the end, while the legacy fallback keeps the original field order.

This keeps retained databases with old SQL signatures from mislabeling all
columns after `leaf_row_object_bytes` when a newer shared library is installed
with `--skip-extension-recreate`.
