# Task 80 AWS 1M Block16 Pruning

Status: partial AWS run complete

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

The AWS run produced a valid 1M `leaf_block_rows=16` / tg256 index and one full
post-repair q500 recall/latency row. The retained AWS extension catalog was
missing `ec_spire_index_scan_leaf_candidate_snapshot(oid, real[])`; this packet
records the catalog-only repair used to register that pgrx function before the
post-repair retry.

Index build:

- `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- size `872 MB` / storage snapshot `872.1 MiB`
- rows `990,000`
- build time `1,704,036 ms`

Completed row, global block cap `1152`, q500:

| nprobe | recall@10 | p50 | p95 | candidates | heap rerank |
| --- | ---: | ---: | ---: | ---: | ---: |
| 96 | 0.9832 | 301.121 ms | 377.482 ms | 9,213,846 | 12,500 |
| 128 | 0.9832 | 320.201 ms | 422.479 ms | 9,213,838 | 12,500 |
| 256 | 0.9832 | 319.523 ms | 417.068 ms | 9,213,838 | 12,500 |

The full matrix did not complete in this run. The post-repair SSM invocation hit
the one-hour execution timeout while the `2048` global cap pipeline was still
running, so `4096` and `8192` were not reached. AWS profile `1m` was paused
after syncing the completed artifacts.

Primary evidence:

- `artifacts/manifest.md`
- `artifacts/aws-1m-block16-pruning-query-after-repair/pipeline-spire-1m-rabitq-block16-global1152.log`
- `artifacts/aws-1m-block16-pruning-query-after-repair/funnel-spire-1m-rabitq-block16-global1152.jsonl`
- `artifacts/aws-1m-block16-pruning-query-after-repair/storage-spire-1m-rabitq-block16-tg256.log`
- `artifacts/ssm-register-candidate-snapshot.json`
- `artifacts/ssm-query-after-repair-timeout.json`
- `artifacts/cloud-status-after-task80-pause.log`
