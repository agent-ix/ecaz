# Task 31 Quality Suite Rerun After IVF Rerank State Cache

Reviewer: please review this packet-local quality-candidate rerun for the
rerank-state-cache checkpoint from `30194`.

## Scope

This packet reruns the Task 31 quality candidate on committed head
`c1a761fd`, which caches heap-f32 rerank fetch state in the IVF scan opaque.

The packet-local suite config covers:

- `recall100-candidates-w1000`
- `latency-candidates-w1000`
- `explain-quality-candidate`

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30195-task31-suite-quality-rerank-state-cache/artifacts/audit.log bench suite audit --config review/30195-task31-suite-quality-rerank-state-cache/task31-m5-ivf-100k-quality.packet.json
/Users/peter/.cargo/bin/ecaz --log-file review/30195-task31-suite-quality-rerank-state-cache/artifacts/install-pg18.log dev install ecaz-pg-test --pg 18
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30195-task31-suite-quality-rerank-state-cache/task31-m5-ivf-100k-quality.packet.json --manifest-output review/30195-task31-suite-quality-rerank-state-cache/artifacts/suite-manifest.json --results-output review/30195-task31-suite-quality-rerank-state-cache/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30195-task31-suite-quality-rerank-state-cache/artifacts/status.log bench suite status --manifest review/30195-task31-suite-quality-rerank-state-cache/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30195-task31-suite-quality-rerank-state-cache/artifacts/report.log bench suite report --manifest review/30195-task31-suite-quality-rerank-state-cache/artifacts/suite-manifest.json --results-output review/30195-task31-suite-quality-rerank-state-cache/artifacts/results.jsonl
```

No cargo or pgrx tests were run in this packet; the code checkpoint validation
is recorded in `30194`.

## Results

Surface:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- quality setting under test: `nprobe=96`, `rerank_width=1000`

Recall@100:

- `nprobe=80`: `0.9880`, mean query time `14.86 ms`
- `nprobe=96`: `0.9920`, mean query time `13.13 ms`

Latency:

- `nprobe=80`: p50 `11.3 ms`, p95 `12.3 ms`, p99 `12.7 ms`
- `nprobe=96`: p50 `12.8 ms`, p95 `13.5 ms`, p99 `13.9 ms`

Threshold status:

- `quality-candidate-recall100-floor`: pass (`0.992 >= 0.99`)
- `quality-candidate-p50-budget-ms`: pass (`12.8 <= 15.0`)

Representative EXPLAIN/counters for `nprobe=96,rerank_width=1000`:

- index bytes: `20291584` (`19 MB`)
- execution time: `15.941 ms`
- selected lists: `96`
- posting pages read: `1815`
- postings visited: `77760`
- postings scored: `2420`
- postings pruned by bound: `75340`
- candidates inserted: `2420`
- rerank rows: `1000`

## Comparison To `30191`

At the same `nprobe=96,rerank_width=1000` quality point:

| metric | `30191` | `30195` |
|---|---:|---:|
| recall@100 | `0.9920` | `0.9920` |
| recall mean q-time | `13.19 ms` | `13.13 ms` |
| latency p50 | `12.8 ms` | `12.8 ms` |
| latency p95 | `13.6 ms` | `13.5 ms` |
| latency p99 | `14.1 ms` | `13.9 ms` |
| explain execution | `16.330 ms` | `15.941 ms` |
| postings visited | `77760` | `77760` |
| postings scored | `2420` | `2420` |
| postings pruned by bound | `75340` | `75340` |
| rerank rows | `1000` | `1000` |

## Interpretation

This is a smaller win than the heap-order fetch checkpoint, but it is aligned
with the intended mechanism:

- recall and thresholds stayed fixed
- scan counters stayed fixed
- quality latency tightened slightly in the tail
- representative explain execution moved down again

The remaining visible cost is no longer obvious setup churn on the rerank path.

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
