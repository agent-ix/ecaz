# Task 31 Balanced Suite Rerun After Heap-Ordered Rerank Fetch

Reviewer: please review this packet-local balanced-candidate rerun for the
heap-ordered rerank-fetch checkpoint from `30190`.

## Scope

This packet reruns the Task 31 balanced candidate on committed head
`79c1a11c`, which fetches heap-f32 rerank rows in heap-TID order before
sorting them back into final score order.

The packet-local suite config covers:

- `recall100-candidates-w500`
- `latency-candidates-w500`
- `explain-balanced-candidate`

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30192-task31-suite-balanced-heap-ordered-rerank/artifacts/audit.log bench suite audit --config review/30192-task31-suite-balanced-heap-ordered-rerank/task31-m5-ivf-100k-balanced.packet.json
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30192-task31-suite-balanced-heap-ordered-rerank/task31-m5-ivf-100k-balanced.packet.json --manifest-output review/30192-task31-suite-balanced-heap-ordered-rerank/artifacts/suite-manifest.json --results-output review/30192-task31-suite-balanced-heap-ordered-rerank/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30192-task31-suite-balanced-heap-ordered-rerank/artifacts/status.log bench suite status --manifest review/30192-task31-suite-balanced-heap-ordered-rerank/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30192-task31-suite-balanced-heap-ordered-rerank/artifacts/report.log bench suite report --manifest review/30192-task31-suite-balanced-heap-ordered-rerank/artifacts/suite-manifest.json --results-output review/30192-task31-suite-balanced-heap-ordered-rerank/artifacts/results.jsonl
```

No cargo or pgrx tests were run in this packet; the code checkpoint validation
is recorded in `30190`.

## Results

Surface:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- balanced setting under test: `nprobe=96`, `rerank_width=500`

Recall@100:

- `nprobe=80`: `0.9639`, mean query time `9.81 ms`
- `nprobe=96`: `0.9676`, mean query time `11.09 ms`

Latency:

- `nprobe=80`: p50 `9.30 ms`, p95 `9.95 ms`, p99 `10.5 ms`
- `nprobe=96`: p50 `10.6 ms`, p95 `11.3 ms`, p99 `11.5 ms`

Representative EXPLAIN/counters for `nprobe=96,rerank_width=500`:

- index bytes: `20291584` (`19 MB`)
- execution time: `13.931 ms`
- selected lists: `96`
- posting pages read: `1815`
- postings visited: `77760`
- postings scored: `1499`
- postings pruned by bound: `76261`
- candidates inserted: `1499`
- rerank rows: `500`

## Comparison To `30188`

At the same `nprobe=96,rerank_width=500` balanced point:

| metric | `30188` | `30192` |
|---|---:|---:|
| recall@100 | `0.9676` | `0.9676` |
| recall mean q-time | `10.98 ms` | `11.09 ms` |
| latency p50 | `10.7 ms` | `10.6 ms` |
| latency p95 | `11.4 ms` | `11.3 ms` |
| latency p99 | `11.9 ms` | `11.5 ms` |
| explain execution | `13.281 ms` | `13.931 ms` |
| postings visited | `77760` | `77760` |
| postings scored | `1499` | `1499` |
| postings pruned by bound | `76261` | `76261` |
| rerank rows | `500` | `500` |

## Interpretation

The balanced lane stays consistent with the quality-lane result:

- recall stayed fixed
- scan counters stayed fixed, as expected
- latency improved slightly across `p50`, `p95`, and `p99`
- the win is smaller than the quality lane because `rerank_width=500` already
  paid less heap-f32 cost

## Artifacts

- `artifacts/audit.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/status.log`
- `artifacts/report.log`
- `artifacts/recall100_real100k_pqg8_n128_w500_p80_96.log`
- `artifacts/latency_real100k_pqg8_n128_w500_p80_96.log`
- `artifacts/explain_real100k_pqg8_n128_p96_w500.sql`
- `artifacts/explain_real100k_pqg8_n128_p96_w500.log`
- `artifacts/truth_real100k_n128_k100.json`
- `artifacts/manifest.md`
