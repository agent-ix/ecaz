# Task 144 Packet 001 Review Request: Geometry Diagnostics

## Summary

This slice starts Task 144 Phase 0 by adding a suite-visible geometry diagnostic
artifact to `ecaz bench spire-pipeline`.

The new `--geometry-output` JSONL contains:

- `spire_geometry_leaf_size_summary`: active leaf assignment distribution,
  including mean/stddev/CV, percentiles, max, and empty leaves.
- `spire_geometry_true_neighbor_concentration`: per-query count of how many
  active leaf lists contain the exact top-k truth rows under current
  single-assignment SPIRE.

`ecaz bench suite` now accepts `geometry_output` on `spire-pipeline` steps, rewrites
`${artifact_dir}`, includes it in expected artifacts, and expands it to
`--geometry-output`.

## Validation

- `cargo test -p ecaz-cli spire_pipeline --no-default-features`
- Result: `30 passed; 0 failed; 0 ignored; 409 filtered out`
- Log: `artifacts/cargo-test-ecaz-cli-spire-pipeline.log`

## Review Focus

Please review whether this is the right Phase 0 measurement surface for the
single-assignment half of Task 144:

- leaf-size variance comes from `ec_spire_index_leaf_snapshot`;
- true-neighbor concentration comes from exact truth plus
  `ec_spire_index_leaf_target_assignment_snapshot`;
- suite support keeps future 50k/100k evidence inside `ecaz bench suite`.

This does not yet implement closure assignment, ratio pruning, or closure
simulation. Those remain follow-on slices.
