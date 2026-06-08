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
- AWS 1M/q500 enriched baseline completed and reproduced the retained Task 83
  surface: `recall@10=0.9832`, `candidate_sum=9,213,846`, p50 `282.869 ms`,
  p95 `354.557 ms`, p99 `368.232 ms`.
- AWS `1m` was paused after the run; final status shows `state: paused`.

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

## AWS Result

The enriched baseline preserved the Task 83 attribution:

- `4916` hits.
- `3` `routing_miss`.
- `81` `selected_leaf_block_pruning_or_candidate_cap`.
- Target context status: `4916` selected by cap, `81` ranked outside cap, `3`
  not found in routed leaves.

For the `81` selected-leaf misses, the packet includes
`artifacts/selected-leaf-miss-enriched-context.tsv`. Key distributions:

- Rank deltas beyond `global1152`: `7` within `+128`, `23` in `+129..+512`,
  `28` in `+513..+2048`, `23` farther than `+2048`.
- Score margin to cap (`block_ip - cap_block_ip`): min `-0.07606012`, p50
  `-0.017314821`, p90 `-0.0036982894`, max `-0.00009295344`.
- Exclusive margin buckets: `3` within `0.001` below cap, `8` in
  `0.001..0.005`, `15` in `0.005..0.01`, `55` worse than `-0.01`.
  Cumulatively, `11` are within `0.005` and `26` are within `0.01`.
- Route rank buckets: `28` in top 8 routes, `24` in routes 9-24, `19` in
  routes 25-48, `10` after route 48.

This points the next Task 84 slice toward block-score calibration inside
already-good routed leaves. A pure near-cap rescue window would only cover a
small part of the `81` misses unless it grows wide enough to look like the
rejected blanket-cap sweep.

## Requested Review

Please review the diagnostic extension for correctness and whether these fields
are sufficient for the next Task 84 block-scoring recovery slice.
