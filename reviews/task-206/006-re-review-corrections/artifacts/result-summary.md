# Corrected Task 206 seed A/B

The attribution-feature PG18 build completed the pre-registered physical
persisted-head A/B at BW64/H8, top-k 200, with effective seed counts 128 and
200. Values below are from `run/results.jsonl`; each arm used the same
three-owner fixture and corpus/query prefix at the stated scale.

| scale | arm | recall | mean latency | p50 | p95 | p99 | max |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | feature-k128 | 0.9883 | 230.90 ms | 227.80 ms | 298.10 ms | 321.10 ms | 330.60 ms |
| 10k | feature-k200 | 0.9870 | 218.60 ms | 207.60 ms | 294.80 ms | 314.00 ms | 318.00 ms |
| 50k | feature-k128 | 0.9600 | 300.10 ms | 308.00 ms | 325.40 ms | 339.70 ms | 350.60 ms |
| 50k | feature-k200 | 0.9600 | 309.50 ms | 315.90 ms | 356.00 ms | 396.30 ms | 427.40 ms |
| 100k | feature-k128 | 0.9584 | 298.30 ms | 307.70 ms | 340.60 ms | 355.60 ms | 361.20 ms |
| 100k | feature-k200 | 0.9587 | 300.70 ms | 310.80 ms | 334.40 ms | 341.20 ms | 344.80 ms |

Physical storage is invariant across the two seed arms: 242,745,344 bytes
and 1.235467 amplification at 10k; 1,242,742,784 bytes and 1.332667 at
50k; 2,496,643,072 bytes and 1.351147 at 100k.

This is a corrected seed-control and observability diagnostic. The feature
build enables attribution instrumentation, so its latency is not comparable
with the clean release matrix and does not promote a production default.
The shipped default remains BW4/H100; BW64/H8 remains a separate
productionization decision.
