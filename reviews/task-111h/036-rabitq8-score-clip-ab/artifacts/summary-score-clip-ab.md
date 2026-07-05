# Task 111h RaBitQ8 Score/Clip A/B Summary

Head SHA under measurement: `53caaa57245763970452425c55a4738a18bc93fd`.

Matrix:
- Corpus: `data/staged-current/ec_real_100k_corpus.tsv`, 100000 rows, dim 1536.
- Queries: `data/staged-current/ec_real_100k_queries.tsv`, 200-query recall/latency subset.
- Index reloptions held constant except score/clip: `storage_format=coarse_rerank`, `coarse_bits=1`, `rerank_placement=index`, `rerank_format=rabitq8`, `rerank_width=64`, `nlists=256`, `nprobe` swept.
- Score modes: `estimator` is `rabitq_rerank_least_squares=0`; `least_squares` is `rabitq_rerank_least_squares=1`.
- Clip values: `rabitq_rerank_clip=2`, `3`, `4`.
- Storage artifact rows are for the `ec_ivf` index only.

The initial all-in-one suite completed both clip=2 variants and failed at `load-100k-index-rabitq8-est-c3-w64` because `/tmp` ran out of space. The remaining four variants were rerun as isolated one-variant continuations, dropping and recreating the benchmark database between variants.

| Score mode | Clip | Recall@10 nprobe32 | Recall@10 nprobe200 | Latency nprobe32 p50/p95/p99 | Latency nprobe200 p50/p95/p99 | Index size |
| --- | ---: | ---: | ---: | --- | --- | --- |
| estimator | 2 | 0.9060 | 0.9525 | 4.34 ms / 5.01 ms / 5.42 ms | 14.5 ms / 16.6 ms / 18.8 ms | 183.6 MiB |
| least_squares | 2 | 0.9050 | 0.9510 | 4.23 ms / 4.95 ms / 5.47 ms | 14.3 ms / 15.7 ms / 20.8 ms | 183.6 MiB |
| estimator | 3 | 0.9260 | 0.9830 | 4.56 ms / 5.44 ms / 6.27 ms | 15.1 ms / 17.3 ms / 22.4 ms | 183.6 MiB |
| least_squares | 3 | 0.9250 | 0.9825 | 4.13 ms / 4.92 ms / 5.46 ms | 14.3 ms / 15.5 ms / 18.3 ms | 183.6 MiB |
| estimator | 4 | 0.9305 | 0.9915 | 4.08 ms / 4.84 ms / 5.69 ms | 14.3 ms / 15.6 ms / 18.7 ms | 183.6 MiB |
| least_squares | 4 | 0.9305 | 0.9920 | 4.25 ms / 5.00 ms / 5.46 ms | 14.5 ms / 16.0 ms / 21.5 ms | 183.6 MiB |

Interpretation:
- The old clip=2 evidence was not enough to judge RaBitQ8 rerank quality. Moving to clip=3 raised nprobe200 recall from about `0.952` to about `0.983`.
- Clip=4 raised the high-probe ceiling further to `0.9915`-`0.9920`, with the same `183.6 MiB` index size and similar warm latency.
- Least-squares scoring did not improve clip=2 or clip=3 recall in this run. At clip=4 it only improved nprobe200 by `0.0005` and did not improve nprobe32.

Source result files:
- `results-report-after-enospc.jsonl` for both clip=2 variants.
- `results-cont-est-c3-report.jsonl` for estimator clip=3.
- `results-cont-ls-c3-report.jsonl` for least-squares clip=3.
- `results-cont-est-c4-report.jsonl` for estimator clip=4.
- `results-cont-ls-c4-report.jsonl` for least-squares clip=4.
