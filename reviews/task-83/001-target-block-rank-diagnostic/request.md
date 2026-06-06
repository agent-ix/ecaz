# Task 83 Review Request: Target-Block Rank Diagnostic

## Summary

This packet adds and validates the Task 83 target-only selected-block
containment diagnostic needed after Task 82 found `81/84` missed truth rows
inside selected leaves.

New surfaces:

- `ec_spire_index_scan_leaf_target_block_rank_snapshot(...)`
- `ecaz bench spire-pipeline --target-block-rank-output`
- `ecaz bench suite` field `target_block_rank_output`

The diagnostic returns the same block-rank columns as the existing full helper,
but it locates target rows in routed leaves first and emits only each target's
containing block rank. That is intended to make AWS q500 attribution feasible
without the Task 82 full helper's all-block row scan.

## AWS Result

The AWS 1M/q500 run completed after the flag-decoding fix in `77cafdacd`.

- Baseline retained surface: `recall@10=0.9832`, `candidate_sum=9,213,846`,
  p50 `288.769 ms`, p95 `363.138 ms`, p99 `375.732 ms`.
- Miss attribution: `84` missed truth rows; `3` pure routing misses and `81`
  selected-leaf block-pruning/candidate-budget misses.
- Target block rank status: `4,916` selected by the global cap, `81` ranked
  outside the cap, `3` not found in routed leaves.
- All `81` selected-leaf misses had `block_rank > 1152` and
  `selected_by_global_cap=false`.
- Selected-leaf miss deltas beyond cap: `7` within `+128`, `30` within `+512`,
  `58` within `+2048`, and `23` farther than `+2048`.

## Validation

- `cargo test -p ecaz-cli spire_pipeline --no-default-features`: passed, `20/20`.
- `cargo build -p ecaz-cli --no-default-features`: passed with only the known `LoadedDistributedPlacementConfig.path` warning.
- Task 83 suite audit passed with `2 steps`.
- AWS `1m` final status after the successful run: `paused`.

See `artifacts/manifest.md` for exact artifact paths.

## Requested Review

Please review the diagnostic implementation and AWS attribution evidence.
Packet `reviews/task-83/002-global-cap-recovery-sweep/` carries the follow-up
recovery sweep and closeout decision.
