# Review Request: Task 79 RaBitQ Global Block Pruning Benchmark

## Summary

This packet benchmarks commit `fc2b6ca022ba9e6384807ea2c791c6a784b4a034` using the RaBitQ-primary Task 79 fixture. It tests `ec_spire.leaf_block_pruning_max_global_blocks` at 0, 384, 400, 512, 768, and 1024 global blocks/query on the same block64 V3 summary index shape as packets 011 and 013.

Result: **negative for Task 79 success gates**.

Global allocation improves the tradeoff compared with the prior per-leaf selectors, but the recall floor is only reached after candidate count is already too high:

| Step | Candidate sum | p50 latency | recall@10 |
| --- | ---: | ---: | ---: |
| global0 baseline | 15,506,227 | 62.122 ms | 0.9975 |
| global384 | 4,684,566 | 43.182 ms | 0.9675 |
| global400 | 4,882,003 | 43.486 ms | 0.9710 |
| global512 | 6,269,044 | 47.527 ms | 0.9860 |
| global768 | 9,444,236 | 55.580 ms | 0.9925 |
| global1024 | 12,634,733 | 63.926 ms | 0.9970 |

## Validation

Artifacts are under `reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/`.

- suite audit passed: 8 steps
- suite dry-run manifest written
- PG18 extension installed with backend SHA256 `6a2b4a329061ce35791c9d500aa63ac133a595abb4fa989917522eee40a48969`
- suite status: completed 8, failed 0, missing artifacts 0, stale 0
- suite report written to `artifacts/report-results.jsonl`

## Interpretation

The allocation strategy is better, but one mean summary per block is still too weak. At budgets that satisfy the candidate gate, recall is 0.9675-0.9710. At the first recall-floor point, candidate count is 9.44M and p50 is 55.58 ms, so both the candidate and latency gates fail.

This should be accepted as negative evidence for "global allocation alone" and as justification for the next direct candidate-reduction attempt: either a two-stage tiny row-sample probe within selected summary blocks, or a multi-representative summary format.

## Review Focus

- Check that packet 015 correctly applies the Task 79 gates to the global selector frontier.
- Check that the comparison to packets 011/013 is fair: same RaBitQ n128/f8/top-graph96/block64 fixture and 200-query recall path.
- Check the conclusion that failure now points to summary semantics, not merely budget allocation.
