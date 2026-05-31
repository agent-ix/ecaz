# Task 75 Intel Local Summary

Run head SHA: `4f6de38964403a415a9a5b26cd0d71ec305914bb`

Suite command:

```bash
target/debug/ecaz bench suite run --config benchmarks/task75-intel-local-routing-envelope/suite.json --database task75_spire_gate --host /home/peter/.pgrx --port 28818 --manifest-output benchmarks/task75-intel-local-routing-envelope/artifacts/suite-manifest.json --log-file benchmarks/task75-intel-local-routing-envelope/artifacts/suite-run-rerun-port28818.log
```

## SPIRE Routing Envelope

| Point | nprobe | recall@10 | p50 | p95 | leaf routes | candidates | per-query candidate p50 | per-query candidate p95 | retained | returned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tg16 b0 | 16 | 0.8525 | 26.814 ms | 33.414 ms | 2,666 | 2,087,914 | 11,990 | 14,609 | 5,000 | 2,000 |
| tg32 b0 | 32 | 0.9310 | 48.199 ms | 54.407 ms | 3,533 | 2,769,013 | 16,061 | 26,484 | 5,000 | 2,000 |
| tg64 b0 | 64 | 0.9825 | 90.643 ms | 100.316 ms | 3,556 | 2,784,952 | 16,061 | 27,089 | 5,000 | 2,000 |
| tg96 b0 | 96 | 0.9975 | 131.292 ms | 143.238 ms | 3,556 | 2,784,952 | 16,061 | 27,089 | 5,000 | 2,000 |
| tg128 b0 | 96 | 0.9975 | 134.271 ms | 145.134 ms | 3,556 | 2,784,952 | 16,061 | 27,089 | 5,000 | 2,000 |

## IVF Control

| Point | recall@10 | mean q-time | p50 | p95 | estimated candidates | observed postings visited | observed postings scored | rerank rows |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| IVF nprobe96 | 0.9980 | 37.85 ms | 37.0 ms | 42.0 ms | 75,000 | 77,760 | 1,499 | 500 |

## Readout

- SPIRE reaches the high-recall target locally at tg96/tg128 b0 with recall@10 `0.9975`.
- The high-recall SPIRE point is slower than the IVF control on this host: about `131-134 ms` p50 vs `37 ms` p50.
- Candidate fan-in saturates by tg64/tg96. The extra nprobe budget no longer increases leaf candidate exposure, but latency still rises with the routing/read path.
- The new funnel JSONL confirms the local gate: roughly `2.78M` leaf candidates are scanned across 200 queries, `5,000` survive to heap rerank, and `2,000` rows are returned.
