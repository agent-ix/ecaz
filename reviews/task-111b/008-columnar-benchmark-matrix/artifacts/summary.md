# Task 111b Columnar Benchmark Matrix Summary

Measured head: `376da5eba72d1e1abe44e86399bd9c32fe8badbf`

Suite: `task111b-columnar-benchmark-matrix`

Status: 50 completed, 0 failed, 0 skipped, 0 missing artifacts, 0 stale.

This packet measured the Task 111b columnar frozen-list format on the same
real-corpus fixtures and nprobe sweep used by Task 111a:

- 50k and 100k fixtures.
- TurboQuant plus RaBitQ quant bits 1, 2, 4, and 8.
- `nlists=64`, index/default `nprobe=32`, `training_sample_rows=10000`.
- `dense_posting_blocks=0`, `columnar_frozen_lists=1`.
- Scan GUCs: `ec_ivf.dense_posting_coalescing=on`,
  `ec_ivf.dense_posting_typed_views=off`.
- One isolated table/index prefix per scale/quant cell.

The first full run exposed a raw-page capacity bug in the columnar writer:
`ec_ivf columnar page payload 8166 exceeds raw capacity 8160`. Commit
`9cdff9976` fixes the chunker to use the guarded raw-page capacity. Focused
raw-page tests passed before the final suite run.

## nprobe=32 Results

| scale | quant | recall@10 | ndcg@10 | latency mean | p50 | p95 | p99 | index size | bytes/row |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | TQ | 0.9420 | 0.9994 | 14.4 ms | 14.3 ms | 15.5 ms | 16.5 ms | 43.4 MiB | 911.1 |
| 50k | rb1 | 0.7750 | 0.9896 | 6.71 ms | 6.60 ms | 7.43 ms | 10.6 ms | 14.2 MiB | 297.0 |
| 50k | rb2 | 0.8840 | 0.9974 | 65.4 ms | 59.9 ms | 89.0 ms | 92.1 ms | 23.9 MiB | 501.5 |
| 50k | rb4 | 0.9410 | 0.9993 | 18.8 ms | 18.2 ms | 23.5 ms | 27.0 ms | 43.4 MiB | 911.1 |
| 50k | rb8 | 0.9460 | 0.9994 | 15.7 ms | 15.4 ms | 18.2 ms | 20.3 ms | 82.5 MiB | 1730.2 |
| 100k | TQ | 0.9370 | 0.9966 | 35.9 ms | 35.7 ms | 43.1 ms | 44.4 ms | 83.4 MiB | 874.4 |
| 100k | rb1 | 0.7630 | 0.9875 | 13.2 ms | 13.1 ms | 14.7 ms | 15.8 ms | 24.8 MiB | 260.1 |
| 100k | rb2 | 0.8670 | 0.9946 | 131.9 ms | 132.4 ms | 145.5 ms | 151.5 ms | 44.4 MiB | 465.2 |
| 100k | rb4 | 0.9290 | 0.9965 | 39.2 ms | 39.3 ms | 44.4 ms | 46.0 ms | 83.4 MiB | 874.4 |
| 100k | rb8 | 0.9390 | 0.9967 | 45.6 ms | 42.7 ms | 63.6 ms | 90.6 ms | 161.5 MiB | 1693.5 |

## EXPLAIN Counters

All rows are `EXPLAIN (ANALYZE, FORMAT JSON)` at nprobe 32.

| scale | quant | posting pages read | columnar lists | postings visited | logical bytes copied | coalesced flushes | scan elapsed |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | TQ | 2630 | 32 | 23904 | 18887163 | 109 | 22533 us |
| 50k | rb1 | 841 | 32 | 23904 | 5405307 | 109 | 6808 us |
| 50k | rb2 | 1437 | 32 | 23904 | 9994875 | 109 | 72912 us |
| 50k | rb4 | 2630 | 32 | 23904 | 19174011 | 109 | 19715 us |
| 50k | rb8 | 5022 | 32 | 23904 | 37532283 | 109 | 21433 us |
| 100k | TQ | 4499 | 32 | 42171 | 33320379 | 178 | 30684 us |
| 100k | rb1 | 1336 | 32 | 42171 | 9535935 | 178 | 10706 us |
| 100k | rb2 | 2393 | 32 | 42171 | 17632767 | 178 | 108137 us |
| 100k | rb4 | 4499 | 32 | 42171 | 33826431 | 178 | 35422 us |
| 100k | rb8 | 8716 | 32 | 42171 | 66213759 | 178 | 61660 us |

## Comparison Against 111a

Recall matches the 111a storage surfaces for each scale/quant/nprobe cell.
Columnar therefore preserves correctness for the measured fixed fixtures.

Storage is better than row for all measured bit widths, but still generally
worse than original dense (`dense-old` / `dense-a`) and usually better than or
near the page-spanning dense-b format:

| scale | quant | columnar | row | dense-old/a | dense-b |
| --- | --- | ---: | ---: | ---: | ---: |
| 50k | TQ/rb4 | 43.4 MiB | 44.1 MiB | 39.8 MiB | 49.2 MiB |
| 50k | rb1 | 14.2 MiB | 15.2 MiB | 11.6 MiB | 12.9 MiB |
| 50k | rb2 | 23.9 MiB | 25.2 MiB | 21.3 MiB | 25.0 MiB |
| 50k | rb8 | 82.5 MiB | 98.4 MiB | 78.9 MiB | 86.0 MiB |
| 100k | TQ/rb4 | 83.4 MiB | 87.6 MiB | 78.9 MiB | 98.1 MiB |
| 100k | rb1 | 24.8 MiB | 29.7 MiB | 22.5 MiB | 25.1 MiB |
| 100k | rb2 | 44.4 MiB | 49.6 MiB | 41.8 MiB | 49.4 MiB |
| 100k | rb8 | 161.5 MiB | 196.0 MiB | 157.0 MiB | 171.4 MiB |

Latency is mixed:

- TQ columnar is close to row but slower than 111a dense-a/dense-b at both
  50k and 100k.
- rb1 columnar is close to 111a row/dense-a at 50k and 100k.
- rb2 columnar repeats the 111a rb2 behavior: the kernel is slow despite wide
  batches. Columnar gets wide coalesced flushes, so this is not a small-batch
  regression.
- rb4 columnar is close to row/dense-b and slower than the best dense-a /
  dense-typed rows.
- rb8 columnar is faster than row at 50k but slower than the best dense rows;
  at 100k it is near row and slower than original dense.

## Interpretation

The columnar proof is functionally valid and keeps metadata once per frozen
list instead of repeating full metadata on every physical continuation tuple.
It also feeds the coalesced scorer with wide batches (`109` flushes at 50k,
`178` at 100k), so the earlier TQ small-batch regression is fixed for this
format.

The storage result is still not the durable winner. It improves over row, but
original dense remains smaller in every measured cell, and the latency winner
is still workload/bit-width dependent. The main next direction is the 111c/111d
layout work: keep logical scorer-width groups, avoid repeated metadata, and
make physical continuation/page layout cheaper without losing the wide-batch
scan shape.

