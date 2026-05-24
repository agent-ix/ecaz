# Task 59 Packet 003: Frontier Heap State

## Summary

Code commit under review:

- `d37d086720b4833677dabdeb25d7eb27f2e76904`
  (`Streamline DiskANN frontier heap state`)

This checkpoint removes the separate `HashMap<ItemPointer, FrontierEntry>` from
DiskANN greedy descent. The heap now owns each `FrontierEntry` directly,
including its neighbor list, while the existing scan scratch sets still enforce
the same invariants:

- `in_frontier` prevents inserting the same TID twice;
- `visited` prevents expanding an already expanded TID;
- candidate ordering is still driven by `ScanCandidate::cmp`;
- traversal, prefilter, exact rerank, and result ordering are unchanged.

## Benchmark Evidence

Owning benchmark packet:

- `benchmarks/task59-aws-diskann-frontier-heap/`

Focused AWS Graviton results on profile `10k` (`m8g.large`), shared retained
Task 55 10k/100k tables:

| dataset | list_size | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 800 | 0.9975 | 3.42 ms | 3.35 ms | 4.38 ms | 4.60 ms |
| 100k | 800 | 0.9865 | 10.2 ms | 10.2 ms | 12.0 ms | 12.9 ms |

Against Task 55 optimized 100k latency:

- `list_size=128`: mean `2.60 -> 2.54 ms`, p95 `3.18 -> 3.06 ms`
- `list_size=200`: mean `3.49 -> 3.45 ms`, p95 `4.27 -> 4.18 ms`
- `list_size=400`: mean `5.88 -> 5.90 ms`, p95 `7.02 -> 7.20 ms`
- `list_size=800`: mean `10.6 -> 10.2 ms`, p95 `12.4 -> 12.0 ms`

Recall rows are unchanged from Task 55 for every 10k/100k `list_size` cell in
the focused suite.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,pg_test`
  passed; see `artifacts/cargo-check-pg18-pg-test.log`.

## Notes

This is a modest traversal-state optimization, not the closing Task 59 result.
The final suite still needs the settled Graviton profile/config and 10k/50k/100k/1M coverage.
