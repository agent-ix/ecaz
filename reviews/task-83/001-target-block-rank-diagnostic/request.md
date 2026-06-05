# Task 83 Review Request: Target-Block Rank Diagnostic

## Summary

This checkpoint adds the Task 83 target-only selected-block containment
diagnostic needed after Task 82 found `81/84` missed truth rows inside selected
leaves.

New surfaces:

- `ec_spire_index_scan_leaf_target_block_rank_snapshot(...)`
- `ecaz bench spire-pipeline --target-block-rank-output`
- `ecaz bench suite` field `target_block_rank_output`

The diagnostic returns the same block-rank columns as the existing full helper,
but it locates target rows in routed leaves first and emits only each target's
containing block rank. That is intended to make AWS q500 attribution feasible
without the Task 82 full helper's all-block row scan.

## Validation

- `cargo test -p ecaz-cli spire_pipeline --no-default-features`: passed, `20/20`.
- `cargo build -p ecaz-cli --no-default-features`: passed with only the known `LoadedDistributedPlacementConfig.path` warning.
- Task 83 suite audit passed with `2 steps`.

See `artifacts/manifest.md` for exact artifact paths.

## AWS Status

The AWS q500 diagnostic run is not complete yet. Two `cloud install` attempts on
the `1m` profile remained silent before writing install log output; both were
stopped locally and AWS was paused. Final status artifact shows `state: paused`.

This packet is therefore a code/runner checkpoint, not the final Task 83
measurement packet.

## Requested Review

Please review the diagnostic implementation for correctness and whether the
target-only block-rank method is an acceptable replacement for the full
block-rank helper before we retry the AWS q500 measurement.
