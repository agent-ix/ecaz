# Task 82 Review Request: AWS 1M Recall Attribution

## Summary

This packet closes Task 82 with bounded AWS 1M/q500 miss attribution for the retained Task 79/81 SPIRE surface.

The measured retained shape remains `recall@10=0.9832` with `candidate_sum=9,213,846`. The attribution output covers all `5,000` q500 truth rows:

| Stage | Missed truth rows |
| --- | ---: |
| `routing_miss` | 3 |
| `selected_leaf_block_pruning_or_candidate_cap` | 81 |
| `assignment_missing` | 0 |
| `candidate_or_rerank_cap` | 0 |

The result points the next implementation slice at selected-leaf block containment and block scoring/pruning recovery. It does not justify a wider routing/top-graph slice yet: only `3/84` misses are pure routing misses, while the prior recall-ceiling top-graph surface recovered recall by expanding to `251M-495M` q500 candidates.

## Implementation Notes

Task 82 added a bounded diagnostic path:

- `bench spire-pipeline --miss-attribution-output`
- suite config support for `miss_attribution_output`
- `ec_spire_index_leaf_target_assignment_snapshot(index_oid, target_local_sequences)` to scan leaf assignments once for only the q500 truth targets

The final run intentionally does not use full `ec_spire_index_scan_leaf_block_rank_snapshot` for q500. That full-rank helper was attempted and cancelled after one backend ran longer than 11 minutes. The bounded diagnostic answers the task-level question without making q500 attribution depend on an unusably slow full block-rank scan.

## Evidence

- Manifest: `reviews/task-82/001-aws-1m-recall-attribution/artifacts/manifest.md`
- Suite config: `reviews/task-82/001-aws-1m-recall-attribution/suite-aws-1m-miss-attribution-q500.json`
- Parsed summary: `reviews/task-82/001-aws-1m-recall-attribution/artifacts/miss-attribution-summary.txt`
- Structured results: `reviews/task-82/001-aws-1m-recall-attribution/artifacts/aws-1m-miss-attribution-q500/results.jsonl`
- Attribution JSONL: `reviews/task-82/001-aws-1m-recall-attribution/artifacts/aws-1m-miss-attribution-q500/miss-attribution-spire-1m-global1152-q500.jsonl`
- Pipeline log: `reviews/task-82/001-aws-1m-recall-attribution/artifacts/aws-1m-miss-attribution-q500/pipeline-spire-1m-rabitq-block-summary-global1152-miss-attribution-q500.log`
- Final AWS status: `reviews/task-82/001-aws-1m-recall-attribution/artifacts/cloud-status-final-paused.log`

## Validation

- `cargo test -p ecaz-cli spire_pipeline --no-default-features` passed.
- `cargo build -p ecaz-cli --no-default-features` passed with only the pre-existing `LoadedDistributedPlacementConfig.path` dead-code warning.
- Suite audit passed: `[suite:task82-aws-1m-miss-attribution-q500] audit passed: 2 steps`.
- AWS `1m` final state is `paused`.

## Requested Review

Please review whether the bounded attribution method and packet evidence are sufficient to close Task 82, and whether the recommended next slice should be selected-leaf block containment/scoring rather than routing breadth.
