# Review Request: Append-Only Row Segment Snapshot Columns

## Summary

This checkpoint fixes the ABI compatibility issue discovered in packet 014.
The first row-segment implementation inserted two fields into the middle of
`ec_spire_index_scan_leaf_candidate_snapshot`. On retained AWS databases where
`--skip-extension-recreate` keeps the old pgrx SQL signature, that can mislabel
all subsequent tuple positions after `leaf_row_object_bytes`.

The new row-segment fields now append at the end of the snapshot output:

- `leaf_row_segment_read_count`
- `leaf_row_segment_read_bytes`

The CLI decodes the current signature with those fields at the end and keeps a
legacy fallback for old signatures.

## Validation

- `cargo fmt --check` passed with existing stable-rustfmt warnings.
- `cargo test --manifest-path crates/ecaz-cli/Cargo.toml spire_pipeline --locked --offline`
  passed: `21 passed; 0 failed`.

## Follow-Up

To get actual selected row-segment bytes on AWS, install this commit and rerun
the retained 1M/q500 suite. The packet 014 successful row remains valid for
recall/latency/candidate comparison, but not for row-segment byte evidence.
