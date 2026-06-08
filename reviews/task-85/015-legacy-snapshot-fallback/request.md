# Review Request: Legacy Leaf Snapshot Fallback

## Summary

This checkpoint fixes the AWS failure recorded in packet 014. The retained AWS
database can preserve its 1M corpus/index with `--skip-extension-recreate`, but
that also preserves the old pgrx SQL return signature for
`ec_spire_index_scan_leaf_candidate_snapshot`.

The CLI now supports both signatures:

- new signature: reads actual `leaf_row_segment_read_count` and
  `leaf_row_segment_read_bytes`;
- legacy signature: falls back to the old column list and reports those segment
  counters as zero.

## Validation

- `cargo fmt --check` passed with existing stable-rustfmt warnings.
- `cargo test --manifest-path crates/ecaz-cli/Cargo.toml spire_pipeline --locked --offline`
  passed: `21 passed; 0 failed`.

## Follow-Up

Rerun the AWS 1M/q500 retained funnel suite from packet 014. If the retained DB
still exposes the legacy signature, the run will not contain actual selected
row-segment bytes, but it will complete and preserve the prior read/score
funnel fields. A later data-preserving extension SQL update can replace the
fallback once available.
