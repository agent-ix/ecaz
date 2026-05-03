# Task 31 Quality Suite Rerun After Heap-Ordered Rerank Fetch

Reviewer: please review this packet-local quality-candidate rerun for the
heap-ordered rerank-fetch checkpoint from `30190`.

## Scope

This packet reruns the Task 31 quality candidate on committed head
`79c1a11c`, which keeps the existing probe/candidate semantics but fetches
heap-f32 rerank rows in heap-TID order before sorting them back into final
score order.

The packet-local suite config covers:

- `recall100-candidates-w1000`
- `latency-candidates-w1000`
- `explain-quality-candidate`

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30191-task31-suite-quality-heap-ordered-rerank/artifacts/audit.log bench suite audit --config review/30191-task31-suite-quality-heap-ordered-rerank/task31-m5-ivf-100k-quality.packet.json
/Users/peter/.cargo/bin/ecaz --log-file review/30191-task31-suite-quality-heap-ordered-rerank/artifacts/install-pg18.log dev install ecaz-pg-test --pg 18
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30191-task31-suite-quality-heap-ordered-rerank/task31-m5-ivf-100k-quality.packet.json --manifest-output review/30191-task31-suite-quality-heap-ordered-rerank/artifacts/suite-manifest.json --results-output review/30191-task31-suite-quality-heap-ordered-rerank/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30191-task31-suite-quality-heap-ordered-rerank/artifacts/status.log bench suite status --manifest review/30191-task31-suite-quality-heap-ordered-rerank/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30191-task31-suite-quality-heap-ordered-rerank/artifacts/report.log bench suite report --manifest review/30191-task31-suite-quality-heap-ordered-rerank/artifacts/suite-manifest.json --results-output review/30191-task31-suite-quality-heap-ordered-rerank/artifacts/results.jsonl
```

Note: an earlier copied-config run mistakenly pointed step-local artifacts at
`30187`. That output is not cited here. The packet was corrected to use
packet-local log, SQL, and truth-cache paths, then rerun cleanly.

No cargo or pgrx tests were run in this packet; the code checkpoint validation
is recorded in `30190`.

## Results

Surface:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- quality setting under test: `nprobe=96`, `rerank_width=1000`

Recall@100:

- `nprobe=80`: `0.9880`, mean query time `12.06 ms`
- `nprobe=96`: `0.9920`, mean query time `13.19 ms`

Latency:

- `nprobe=80`: p50 `11.3 ms`, p95 `12.2 ms`, p99 `12.8 ms`
- `nprobe=96`: p50 `12.8 ms`, p95 `13.6 ms`, p99 `14.1 ms`

Threshold status:

- `quality-candidate-recall100-floor`: pass (`0.992 >= 0.99`)
- `quality-candidate-p50-budget-ms`: pass (`12.8 <= 15.0`)

Representative EXPLAIN/counters for `nprobe=96,rerank_width=1000`:

- index bytes: `20291584` (`19 MB`)
- execution time: `16.330 ms`
- selected lists: `96`
- posting pages read: `1815`
- postings visited: `77760`
- postings scored: `2420`
- postings pruned by bound: `75340`
- candidates inserted: `2420`
- rerank rows: `1000`

## Comparison To `30187`

At the same `nprobe=96,rerank_width=1000` quality point:

| metric | `30187` | `30191` |
|---|---:|---:|
| recall@100 | `0.9920` | `0.9920` |
| recall mean q-time | `13.08 ms` | `13.19 ms` |
| latency p50 | `13.1 ms` | `12.8 ms` |
| latency p95 | `14.2 ms` | `13.6 ms` |
| latency p99 | `15.1 ms` | `14.1 ms` |
| explain execution | `16.070 ms` | `16.330 ms` |
| postings visited | `77760` | `77760` |
| postings scored | `2420` | `2420` |
| postings pruned by bound | `75340` | `75340` |
| rerank rows | `1000` | `1000` |

## Interpretation

This checkpoint appears to convert the earlier scan-side pruning win into a
cleaner quality-lane latency result:

- recall and threshold pass/fail stayed fixed
- scan counters stayed fixed, as expected, because this change only reorders
  heap-f32 rerank fetches
- quality latency improved modestly but consistently across `p50`, `p95`, and
  `p99`
- explain execution stayed in the same band, so the visible improvement is
  specifically in the benchmarked rerank-heavy path, not a change in scan work

That makes `79c1a11c` a stronger Task 31 checkpoint than `422e5ddd` alone.

## Next Checkpoint

The next useful slice is to rerun the balanced candidate on this checkpoint and
then restage the decision packet with both score-ranked-probe and
heap-ordered-rerank effects folded in.

## Artifacts

- `artifacts/audit.log`
- `artifacts/install-pg18.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/status.log`
- `artifacts/report.log`
- `artifacts/recall100_real100k_pqg8_n128_w1000_p80_96.log`
- `artifacts/latency_real100k_pqg8_n128_w1000_p80_96.log`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.sql`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.log`
- `artifacts/truth_real100k_n128_k100.json`
- `artifacts/manifest.md`
