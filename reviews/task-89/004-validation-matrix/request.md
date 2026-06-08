# Task 89 / Packet 004: Validation Matrix

## Summary

This packet defines the all-AM validation matrix Task 89 needs after the TQ+
ports land. It also adds a post-port real10k suite template showing how the
ADR-076 reloption surface should be represented in `ecaz bench suite`.

No code porting is included. The suite template is not expected to run on the
current branch because `turboquant_profile` and DiskANN `storage_format =
'turboquant'` do not exist yet.

## Artifacts

- `artifacts/task89-validation-matrix.md`
- `task89-all-am-real10k-template.json`

## Validation

Documentation/template checkpoint only. No suite was run.

Source inspection covered:

- `crates/ecaz-cli/src/commands/bench/suite.rs`
- `crates/ecaz-cli/suites/profile-cross-engine-real10k.json`
- `crates/ecaz-cli/suites/profile-hnsw-100k.json`
- `crates/ecaz-cli/suites/profile-ivf-100k.json`
- `reviews/task-86/008-spire-real-spread/suite-lutoff.json`
- `reviews/task-86/008-spire-real-spread/suite-luton.json`

## Reviewer Focus

Please check whether the matrix is sufficient for Task 89 closeout:

1. DBPedia 10k/50k/100k for IVF, SPIRE, HNSW, and DiskANN.
2. One non-DBPedia cross-corpus all-AM matrix.
3. Streaming-insert drift with the ADR-076 thresholds.
4. Explicit packet paths required in the final closeout table.
