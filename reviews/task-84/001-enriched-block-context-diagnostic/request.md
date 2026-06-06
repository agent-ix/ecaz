# Task 84 Review Request: Enriched Block Context Diagnostic

## Summary

This checkpoint starts Task 84 with the baseline harness work instead of jumping
directly into another recovery policy. It enriches the existing SPIRE
target-block rank diagnostic so the Task 84 baseline can explain why selected
leaf target blocks miss the retained `global1152` cap.

New JSONL fields emitted by `ecaz bench spire-pipeline --target-block-rank-output`:

- `cap_block_ip`
- `block_ip_margin_to_cap`
- `route_rank`
- `route_score`

The code does not change scan selection behavior. It only exposes context that
already exists during the diagnostic path: the route order/score for the loaded
leaf and the summary-score threshold at the global block cap.

## Validation

- `cargo test -p ecaz-cli spire_pipeline --no-default-features`: passed,
  `20/20`.
- `ecaz bench suite audit`: passed for the Task 84 AWS 1M/q500 enriched
  baseline suite.

## Baseline Suite

The checked-in suite config is:

`reviews/task-84/001-enriched-block-context-diagnostic/suite-aws-1m-enriched-block-context-q500.json`

It reruns the retained Task 79/81 surface on AWS 1M/q500:

- `ec_spire.leaf_block_pruning_max_global_blocks=1152`
- `nprobe=96`
- `rerank_width=25`
- truth cache:
  `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`
- target output:
  `target-block-context-spire-1m-global1152-q500.jsonl`

The suite includes a raw setup step that re-registers
`ec_spire_index_scan_leaf_target_block_rank_snapshot(...)` with the enriched
return columns for AWS instances whose SQL wrapper still has the older Task 83
shape.

## Requested Review

Please review the diagnostic extension for correctness and whether these fields
are sufficient for the first Task 84 baseline packet before we spend AWS time on
the q500 enriched context run.
