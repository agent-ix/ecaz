# Task 31 Quality Suite Rerun After IVF Rerank Loop Cleanup

Reviewer: please review this packet-local quality-candidate rerun for the
rerank-loop cleanup checkpoint from `30198`.

## Scope

This packet reruns the Task 31 quality candidate on committed head
`7b28c43b`, which only removes redundant per-row work inside the IVF heap-f32
rerank loop.

The packet-local suite config covers:

- `recall100-candidates-w1000`
- `latency-candidates-w1000`
- `explain-quality-candidate`

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30199-task31-suite-quality-rerank-loop-cleanup/artifacts/audit.log bench suite audit --config review/30199-task31-suite-quality-rerank-loop-cleanup/task31-m5-ivf-100k-quality.packet.json
/Users/peter/.cargo/bin/ecaz --log-file review/30199-task31-suite-quality-rerank-loop-cleanup/artifacts/install-pg18.log dev install ecaz-pg-test --pg 18
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30199-task31-suite-quality-rerank-loop-cleanup/task31-m5-ivf-100k-quality.packet.json --manifest-output review/30199-task31-suite-quality-rerank-loop-cleanup/artifacts/suite-manifest.json --results-output review/30199-task31-suite-quality-rerank-loop-cleanup/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30199-task31-suite-quality-rerank-loop-cleanup/artifacts/status.log bench suite status --manifest review/30199-task31-suite-quality-rerank-loop-cleanup/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30199-task31-suite-quality-rerank-loop-cleanup/artifacts/report.log bench suite report --manifest review/30199-task31-suite-quality-rerank-loop-cleanup/artifacts/suite-manifest.json --results-output review/30199-task31-suite-quality-rerank-loop-cleanup/artifacts/results.jsonl
```

No cargo or pgrx tests were run in this packet; the code checkpoint validation
is recorded in `30198`.

## Results

Surface:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- quality setting under test: `nprobe=96`, `rerank_width=1000`

Recall@100:

- `nprobe=80`: `0.9880`, mean query time `14.79 ms`
- `nprobe=96`: `0.9920`, mean query time `13.13 ms`

Latency:

- `nprobe=80`: p50 `11.5 ms`, p95 `12.4 ms`, p99 `12.6 ms`
- `nprobe=96`: p50 `12.8 ms`, p95 `13.6 ms`, p99 `13.9 ms`

Threshold status:

- `quality-candidate-recall100-floor`: pass (`0.992 >= 0.99`)
- `quality-candidate-p50-budget-ms`: pass (`12.8 <= 15.0`)

Representative EXPLAIN/counters for `nprobe=96,rerank_width=1000`:

- index bytes: `20291584` (`19 MB`)
- execution time: `16.125 ms`
- selected lists: `96`
- posting pages read: `1815`
- postings visited: `77760`
- postings scored: `2420`
- postings pruned by bound: `75340`
- candidates inserted: `2420`
- rerank rows: `1000`

## Comparison To `30195`

At the same `nprobe=96,rerank_width=1000` quality point:

| metric | `30195` | `30199` |
|---|---:|---:|
| recall@100 | `0.9920` | `0.9920` |
| recall mean q-time | `13.13 ms` | `13.13 ms` |
| latency p50 | `12.8 ms` | `12.8 ms` |
| latency p95 | `13.5 ms` | `13.6 ms` |
| latency p99 | `13.9 ms` | `13.9 ms` |
| explain execution | `15.941 ms` | `16.125 ms` |
| postings visited | `77760` | `77760` |
| postings scored | `2420` | `2420` |
| postings pruned by bound | `75340` | `75340` |
| rerank rows | `1000` | `1000` |

## Interpretation

This checkpoint is not a useful promotion over `c1a761fd`:

- recall stayed fixed
- scan counters stayed fixed
- quality latency was flat to slightly worse
- explain moved slightly worse

So this is a valid negative-result packet, but not the new Task 31 baseline.

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
