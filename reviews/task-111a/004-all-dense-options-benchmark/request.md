# Review Request: Task 111a All Dense Options Benchmark

## Summary

This packet measures every Task 111a option requested by the latest feedback on
the current branch:

- row postings;
- dense-old: current per-block dense format, no scan coalescing;
- dense-a: current dense format plus scan-time coalescing;
- dense-typed: aligned one-page typed layout, no scan coalescing;
- dense-b: page-spanning logical dense groups;
- dense-b-typed: page-spanning logical dense groups with typed-view reads
  enabled.

Measured head:

- `c543e7a96 Task 111a: sweep quant bit-width; multi-page group efficiency is a primary objective`

The suite completed 120/120 steps on PG18 with no failures, stale artifacts, or
missing artifacts.

## Result

Recall is unchanged across all dense options at each scale/quantization setting.
At nprobe 32/64, every layout variant matches row recall and NDCG.

Latency at nprobe 32 shows the main tradeoff:

| scale | quant | row | dense-old | dense-a | dense-typed | dense-b | dense-b-typed |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | TQ | 15.0 ms | 17.7 ms | 13.1 ms | 22.0 ms | 14.6 ms | 13.8 ms |
| 50k | RaBitQ | 7.29 ms | 6.06 ms | 6.17 ms | 5.99 ms | 7.14 ms | 7.31 ms |
| 100k | TQ | 32.4 ms | 38.4 ms | 28.2 ms | 37.7 ms | 29.0 ms | 30.4 ms |
| 100k | RaBitQ | 14.7 ms | 12.2 ms | 12.6 ms | 12.2 ms | 14.0 ms | 13.9 ms |

Interpretation:

- TQ regressed in dense-old because physical block fragmentation turns each
  scorer-width group into many small width 8-15 kernel flushes. At 100k/nprobe32
  dense-old emitted 521,755 AVX2 flushes for 5.2M candidates, with 519,350 in
  width 8-15 and none at width >=32.
- dense-a fixes that by coalescing blocks back to scorer-width batches at scan
  time. It is the best TQ option in this run: 28.2 ms p50 at 100k/nprobe32,
  versus row 32.4 ms and dense-old 38.4 ms.
- dense-typed alone does not fix the regression. It preserves the same small
  physical batches as dense-old, so typed reads are not enough without wider
  logical grouping/coalescing.
- dense-b mostly restores width >=32 batches and is close for TQ, but current
  page-spanning assembly/storage overhead keeps it behind dense-a in this
  implementation.
- RaBitQ benefits from dense-old/dense-a/dense-typed because its scoring kernel
  is cheaper; the smaller dense posting footprint and lower overhead dominate
  more than the extra flush count. dense-b is slower than dense-a for RaBitQ in
  this run.

Storage:

- dense-old/dense-a/dense-typed reduce index size versus row for both TQ and
  RaBitQ.
- dense-b increases storage versus row for TQ and versus dense-a for RaBitQ in
  the current implementation, so it is not yet the better durable format despite
  matching recall and mostly restoring wide batches.

## Evidence

See `artifacts/manifest.md` for commands, artifact inventory, and cited result
lines. `artifacts/summary.md` contains the compact extracted tables from
`artifacts/suite/results-report.jsonl`.

