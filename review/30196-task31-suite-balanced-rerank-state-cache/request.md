# Task 31 Balanced Suite Rerun After IVF Rerank State Cache

Reviewer: please review this packet-local balanced-candidate rerun for the
rerank-state-cache checkpoint from `30194`.

## Scope

This packet reruns the Task 31 balanced candidate on committed head
`c1a761fd`, which caches heap-f32 rerank fetch state in the IVF scan opaque.

The packet-local suite config covers:

- `recall100-candidates-w500`
- `latency-candidates-w500`
- `explain-balanced-candidate`

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30196-task31-suite-balanced-rerank-state-cache/artifacts/audit.log bench suite audit --config review/30196-task31-suite-balanced-rerank-state-cache/task31-m5-ivf-100k-balanced.packet.json
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30196-task31-suite-balanced-rerank-state-cache/task31-m5-ivf-100k-balanced.packet.json --manifest-output review/30196-task31-suite-balanced-rerank-state-cache/artifacts/suite-manifest.json --results-output review/30196-task31-suite-balanced-rerank-state-cache/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30196-task31-suite-balanced-rerank-state-cache/artifacts/status.log bench suite status --manifest review/30196-task31-suite-balanced-rerank-state-cache/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30196-task31-suite-balanced-rerank-state-cache/artifacts/report.log bench suite report --manifest review/30196-task31-suite-balanced-rerank-state-cache/artifacts/suite-manifest.json --results-output review/30196-task31-suite-balanced-rerank-state-cache/artifacts/results.jsonl
```

No cargo or pgrx tests were run in this packet; the code checkpoint validation
is recorded in `30194`.

## Results

Surface:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- balanced setting under test: `nprobe=96`, `rerank_width=500`

Recall@100:

- `nprobe=80`: `0.9639`, mean query time `9.77 ms`
- `nprobe=96`: `0.9676`, mean query time `10.93 ms`

Latency:

- `nprobe=80`: p50 `9.27 ms`, p95 `10.1 ms`, p99 `10.3 ms`
- `nprobe=96`: p50 `10.7 ms`, p95 `11.6 ms`, p99 `12.1 ms`

Representative EXPLAIN/counters for `nprobe=96,rerank_width=500`:

- index bytes: `20291584` (`19 MB`)
- execution time: `13.281 ms`
- selected lists: `96`
- posting pages read: `1815`
- postings visited: `77760`
- postings scored: `1499`
- postings pruned by bound: `76261`
- candidates inserted: `1499`
- rerank rows: `500`

## Comparison To `30192`

At the same `nprobe=96,rerank_width=500` balanced point:

| metric | `30192` | `30196` |
|---|---:|---:|
| recall@100 | `0.9676` | `0.9676` |
| recall mean q-time | `11.09 ms` | `10.93 ms` |
| latency p50 | `10.6 ms` | `10.7 ms` |
| latency p95 | `11.3 ms` | `11.6 ms` |
| latency p99 | `11.5 ms` | `12.1 ms` |
| explain execution | `13.931 ms` | `13.281 ms` |
| postings visited | `77760` | `77760` |
| postings scored | `1499` | `1499` |
| postings pruned by bound | `76261` | `76261` |
| rerank rows | `500` | `500` |

## Interpretation

This slice does not show the same clean balanced-lane latency win that it shows
on the quality lane:

- recall stayed fixed
- scan counters stayed fixed
- explain moved down again
- the latency table is mixed at `w500`

That is still acceptable for this checkpoint, because the balanced lane already
had much less rerank work and the quality lane is the binding Task 31 target.

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
