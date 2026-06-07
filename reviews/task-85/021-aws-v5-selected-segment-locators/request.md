# Task 85 Packet 021: AWS V5 Selected Segment Locators

## Summary

This packet measures the V5 selected row-segment locator implementation from
packet 020 on AWS 1M/q500.

Result: V5 preserves the retained recall/candidate surface and materially
reduces object-read time, but it does not yet beat the best retained Task 85
end-to-end latency bar. The object-read/layout lever is therefore not a
product-Pareto exit by itself; Task 85 continues with summary scoring CPU as
the next required same-recall latency workstream.

## Key Results

| Run | recall@10 | candidate_sum | heap_rerank_sum | p50 | p95 | p99 | max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| same-suite retained old control | 0.9876 | 9,213,846 | 12,500 | 257.559 ms | 329.660 ms | 2577.758 ms | 27738.801 ms |
| V5 first | 0.9876 | 9,213,846 | 12,500 | 233.653 ms | 289.871 ms | 303.805 ms | 308.862 ms |
| V5 repeat | 0.9876 | 9,213,846 | 12,500 | 233.850 ms | 290.126 ms | 302.307 ms | 307.818 ms |

The same-suite old control had a severe outlier, so the retained packet 019
warm repeat remains the stronger product bar: `227.388/284.166/297.164 ms`
p50/p95/p99. V5 repeat did not beat that bar.

## Funnel Outcome

| Run | object-read p50 | object-read p95 | candidate-score p50 | summary-score p50 | row-score p50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| same-suite retained old control | 196.359 ms | 262.521 ms | 57.758 ms | 47.602 ms | 10.148 ms |
| V5 repeat | 26.855 ms | 27.891 ms | 57.668 ms | 47.597 ms | 10.067 ms |

V5 kept the selected surface identical:

- `selected_blocks_sum=576,000`
- `row_segment_read_count_sum=1,180,606`
- `row_segment_read_bytes_sum=9,622,405,352`
- `leaf_row_object_bytes_sum=304,802,815,448`

That means the implementation removed the legacy read-chain overhead, but the
remaining end-to-end latency is now dominated by candidate/summary scoring and
other fixed query work.

## Build And Storage

- V5 index build total: `1,701,828 ms` (about 28.4 minutes).
- V5 index size: `872.1 MiB`, `923.7 B/row`.
- Retained Task 80 block16 index size in the same storage report:
  `872.1 MiB`, `923.7 B/row`.

## Task Ledger Update

Updated `plan/tasks/85-spire-product-scale-pareto-program.md`:

- object-read and physical layout: rejected as a Task 85 product-Pareto exit,
  despite proving the direct-locator sublever;
- summary scoring CPU: moved to `implementing`;
- benchmark harness evidence: extended through packet 021;
- added an execution rule that identified same-recall latency levers may not
  be deferred as "future research" without packet-local exits.

AWS `1m` final status is captured as paused in
`artifacts/cloud-status-final-after-v5-q500.log`.
