# Task 59 Packet 005: No Visited Checks

## Summary

Code commit under review:

- `af9e874f7c0ad5b979e9d649083f871fe72f591b`
  (`Remove redundant DiskANN scan visited checks`)

This checkpoint removes redundant per-pop `visited` checks from DiskANN greedy
descent after packet 003 made the heap own `FrontierEntry` values directly.

The scan loop still uses `VisitedState::in_frontier` as the single seen-TID set.
Because every neighbor is checked against `in_frontier` before insertion, a TID
can enter the heap only once. That makes the previous `visited` membership check
on every peek/pop redundant in this scan path. The change removes that extra
hash lookup/insert work while preserving traversal order.

## Benchmark Evidence

Owning benchmark packet:

- `benchmarks/task59-aws-diskann-no-visited-checks/`

Focused AWS Graviton results on profile `10k` (`m8g.large`), shared retained
Task 55 10k/100k tables:

| dataset | list_size | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 800 | 0.9975 | 3.50 ms | 3.44 ms | 4.50 ms | 4.87 ms |
| 100k | 800 | 0.9865 | 9.85 ms | 9.98 ms | 11.9 ms | 12.0 ms |

Against Task 55 optimized 100k latency:

- `list_size=64`: mean `1.72 -> 1.63 ms`, p95 `2.21 -> 2.15 ms`
- `list_size=128`: mean `2.60 -> 2.44 ms`, p95 `3.18 -> 2.96 ms`
- `list_size=200`: mean `3.49 -> 3.35 ms`, p95 `4.27 -> 4.04 ms`
- `list_size=400`: mean `5.88 -> 5.61 ms`, p95 `7.02 -> 6.76 ms`
- `list_size=800`: mean `10.6 -> 9.85 ms`, p95 `12.4 -> 11.9 ms`

Recall rows match Task 55 at every 10k/100k `list_size` cell in the focused suite.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,pg_test`
  passed; see `artifacts/cargo-check-pg18-pg-test.log`.

## Notes

This is a meaningful scan-loop cleanup, but it still does not close Task 59.
The final Graviton suite still needs 10k/50k/100k/1M coverage after profile/config selection.
