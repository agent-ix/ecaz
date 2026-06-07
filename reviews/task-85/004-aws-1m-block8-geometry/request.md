# Task 85 Review Request: AWS 1M Block8 Geometry Slice

## Summary

This packet prepares the next Task85 latency mechanism suite: build and measure
a separate `leaf_block_rows=8` SPIRE index at AWS 1M/q500. Packet 003 showed
the retained miss distribution is too broad for another small cap/recovery
sweep, so this slice tests whether smaller leaf blocks reduce candidate density
enough to beat the Task85 warm latency floor while retaining recall.

## Suite

`reviews/task-85/004-aws-1m-block8-geometry/suite-aws-1m-block8-geometry-q500.json`

The suite will:

- precheck the retained block16 and candidate block8 index state;
- build `aws_spire_1m_rabitq_t85_block8_tg256_idx` with
  `ec_spire.leaf_block_rows=8`;
- run q500 `nprobe=96`, `rerank_width=25` rows at global caps `1152`, `1536`,
  `2048`, and `2304`;
- repeat `global2048` as a warm candidate-density row;
- capture storage after build.

The retained Task85 warm floor remains:

| Recall@10 | p50 | p95 | p99 | Candidate Sum |
| ---: | ---: | ---: | ---: | ---: |
| 0.9832 | 246.397 ms | 304.476 ms | 321.342 ms | 9,213,846 |

## Acceptance Bar

This packet can only claim a product-scale latency improvement if an AWS q500
block8 row has:

- `recall@10 >= 0.9832`;
- p50/p95/p99 better than the retained warm row, with enough margin to be
  credible against AWS run variance;
- `candidate_sum < 9,213,846`;
- storage/build impact reported.

## Validation

- `ecaz bench suite audit`: passed for 8 steps.
- AWS run: pending.

## Requested Review

Please review whether this suite is the right block-geometry test for Task85
before using AWS time.
