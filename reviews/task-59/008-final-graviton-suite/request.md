# Review Request: Final AWS Graviton DiskANN Suite

## Summary

This packet closes the Task 59 1M evidence gap for the low-cost Graviton target. The final benchmark evidence is split because the first suite run completed 10k/50k/100k and then failed the 1M load with the wrong chunked-manifest assumption; after suite-driven cleanup of duplicate staging and the partial 1M table, the 1M resume suite completed load, recall, latency, storage, and explain.

The durable benchmark packet is `benchmarks/task59-aws-diskann-final-graviton-suite/`.

## Results

1M DiskANN on `10k-medium` / `m8g.2xlarge`:

| list_size | recall@10 | mean | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 0.9385 | 3.90 ms | 3.72 ms | 5.80 ms | 7.69 ms |
| 128 | 0.9655 | 5.41 ms | 5.20 ms | 8.20 ms | 9.65 ms |
| 200 | 0.9735 | 6.98 ms | 6.68 ms | 10.8 ms | 12.7 ms |
| 400 | 0.9800 | 11.4 ms | 11.3 ms | 18.2 ms | 21.3 ms |
| 800 | 0.9825 | 19.9 ms | 19.7 ms | 30.9 ms | 35.6 ms |

Storage at 1M: `990000` rows, `15.9 GiB` total, `455.1 MiB` DiskANN index, `482.0 B` index bytes/row.

## Evidence

- Benchmark source of truth: `benchmarks/task59-aws-diskann-final-graviton-suite/manifest.md`
- Summary: `benchmarks/task59-aws-diskann-final-graviton-suite/results-summary.md`
- 10k/50k/100k artifact root: `benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/s3-final-224632/`
- 1M artifact root: `benchmarks/task59-aws-diskann-final-graviton-suite/artifacts/one-million-resume/`
- Packet artifact manifest: `artifacts/manifest.md`

## Interpretation

The 1M gate is now closed for Task 59. The supported optimization claim remains conservative: the scan-loop changes are recall-preserving allocation and simplification wins. The focused 100k latency packet and the final 100k suite differ enough that this packet does not claim a proven latency delta for those micro-optimizations.
