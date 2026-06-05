# Task 80 AWS 1M Block16 Pruning

Status: closeout - AWS 1M follow-up measured, Task 80 path shelved

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

The AWS run produced a valid 1M `leaf_block_rows=16` / tg256 index and two full
post-repair q500 recall/latency rows. The retained AWS extension catalog was
missing `ec_spire_index_scan_leaf_candidate_snapshot(oid, real[])`; this packet
records the catalog-only repair used to register that pgrx function before the
post-repair retry.

Index build:

- `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- size `872 MB` / storage snapshot `872.1 MiB`
- rows `990,000`
- build time `1,704,036 ms`

Completed rows, q500:

| global cap | nprobe | recall@10 | p50 | p95 | candidates | heap rerank |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1152 | 96 | 0.9832 | 301.121 ms | 377.482 ms | 9,213,846 | 12,500 |
| 1152 | 128 | 0.9832 | 320.201 ms | 422.479 ms | 9,213,838 | 12,500 |
| 1152 | 256 | 0.9832 | 319.523 ms | 417.068 ms | 9,213,838 | 12,500 |
| 2048 | 96 | 0.9914 | 308.087 ms | 357.431 ms | 16,379,614 | 12,500 |
| 2048 | 128 | 0.9918 | 334.571 ms | 413.553 ms | 16,379,623 | 12,500 |
| 2048 | 256 | 0.9918 | 334.867 ms | 413.172 ms | 16,379,623 | 12,500 |

Decision:

- `global1152` matches the old tg96 recall row (`0.9832`) but is slower than
  the old p50 (`301.121 ms` vs `268.824 ms` at nprobe 96).
- `global2048` materially improves recall (`0.9914` to `0.9918`) but does so by
  raising the q500 scored-candidate surface to about `16.38M` and p50 to
  `308.087-334.867 ms`.
- `4096` and `8192` were not run after the successful `2048` continuation. They
  can only spend more block budget on this same mechanism, while Task 80 needs a
  candidate/latency win, not a larger recall-spend control.

Task 80 is therefore closed as a measured failed latency path. The local 100k
row was credible, but AWS 1M shows this block-cap tuning does not preserve the
candidate-surface win at scale. The next owner is
`plan/tasks/81-spire-leaf-block-summary-format.md`, which should implement the
deeper ADR-074-style persisted leaf block-summary format instead of further
global-cap sweeps.

Primary evidence:

- `artifacts/manifest.md`
- `artifacts/aws-1m-block16-pruning-query-after-repair/pipeline-spire-1m-rabitq-block16-global1152.log`
- `artifacts/aws-1m-block16-pruning-query-after-repair/funnel-spire-1m-rabitq-block16-global1152.jsonl`
- `artifacts/aws-1m-block16-pruning-query-after-repair/storage-spire-1m-rabitq-block16-tg256.log`
- `artifacts/aws-1m-block16-pruning-continuation/pipeline-spire-1m-rabitq-block16-global2048.log`
- `artifacts/aws-1m-block16-pruning-continuation/funnel-spire-1m-rabitq-block16-global2048.jsonl`
- `artifacts/aws-1m-block16-pruning-continuation/results-global2048.jsonl`
- `artifacts/ssm-continuation-global2048-success.json`
- `artifacts/ssm-register-candidate-snapshot.json`
- `artifacts/ssm-query-after-repair-timeout.json`
- `artifacts/ec2-status-after-task80-continuation-pause.log`
