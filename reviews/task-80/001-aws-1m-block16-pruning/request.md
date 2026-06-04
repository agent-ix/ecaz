# Task 80 AWS 1M Block16 Pruning

Status: in progress

## Purpose

This packet captures the AWS 1M follow-up required by Task 80 after the local
Task 79/80 candidate passed the local gate:

- 100k local accepted row: RaBitQ, `leaf_block_rows=16`, global block cap
  `1152`, `nprobe=96`, `rerank_width=25`.
- Local result: recall@10 `0.9940`, p50 `35.293 ms`, p95 `40.600 ms`,
  `3,673,383` scored candidates over 200 queries.
- Baseline for comparison: Task 78 nprobe96 recall@10 `0.9975`, p50
  `60.256 ms`, `15,506,227` scored candidates over 200 queries.

The AWS run rebuilds a single active SPIRE index on the retained 1M corpus with
`leaf_block_rows=16` and `top_graph_search_list_size=256`, then sweeps global
block caps `1152`, `2048`, `4096`, and `8192` at nprobe `96`, `128`, and
`256`.

## Benchmark Command

```sh
target/debug/ecaz cloud bench \
  --profile 1m \
  --database postgres \
  --config reviews/task-80/001-aws-1m-block16-pruning/suite-aws-1m-block16-pruning.json \
  --suite task80-aws-1m-block16-pruning \
  --ecaz-bin /usr/local/bin/ecaz \
  --log-file reviews/task-80/001-aws-1m-block16-pruning/artifacts/cloud-bench-task80-aws-1m-block16-pruning.log
```

## Comparisons

The accepted AWS row should compare against:

- old tg96 1M row from `benchmarks/aws-spire-1m/001-run/`: recall@10
  `0.9832`, p50 `268.824 ms`, `9,213,846` candidates over 500 queries.
- tg256 recall-ceiling rows from
  `benchmarks/aws-spire-1m-topgraph-rebuild/001-run/`: recall@10 up to
  `1.0000`, but with `251,510,240` to `495,000,000` candidates over 500
  queries.

## Results

Pending AWS run.
