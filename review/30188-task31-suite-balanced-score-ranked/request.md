# Task 31 Balanced Suite Rerun After Score-Ranked Probe Order

Reviewer: please review this packet-local balanced-candidate rerun after the
probe-order change from `30186`.

## Scope

This packet reruns the Task 31 balanced candidate on the already-pushed
score-ranked probe-order line. The active code checkpoint is still the
`422e5ddd` implementation commit; `9d1e59b9` only added review packets.

The packet-local suite config covers:

- `recall100-candidates-w500`
- `latency-candidates-w500`
- `explain-balanced-candidate`

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30188-task31-suite-balanced-score-ranked/artifacts/audit.log bench suite audit --config review/30188-task31-suite-balanced-score-ranked/task31-m5-ivf-100k-balanced.packet.json
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30188-task31-suite-balanced-score-ranked/task31-m5-ivf-100k-balanced.packet.json --manifest-output review/30188-task31-suite-balanced-score-ranked/artifacts/suite-manifest.json --results-output review/30188-task31-suite-balanced-score-ranked/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30188-task31-suite-balanced-score-ranked/artifacts/status.log bench suite status --manifest review/30188-task31-suite-balanced-score-ranked/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30188-task31-suite-balanced-score-ranked/artifacts/report.log bench suite report --manifest review/30188-task31-suite-balanced-score-ranked/artifacts/suite-manifest.json --results-output review/30188-task31-suite-balanced-score-ranked/artifacts/results.jsonl
```

No cargo or pgrx tests were run in this packet; the code checkpoint validation
is recorded in `30186`.

## Results

Surface:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- balanced setting under test: `nprobe=96`, `rerank_width=500`

Recall@100:

- `nprobe=80`: `0.9639`, mean query time `9.84 ms`
- `nprobe=96`: `0.9676`, mean query time `10.98 ms`

Latency:

- `nprobe=80`: p50 `9.50 ms`, p95 `11.1 ms`, p99 `12.1 ms`
- `nprobe=96`: p50 `10.7 ms`, p95 `11.4 ms`, p99 `11.9 ms`

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

## Comparison To `30184`

At the same `nprobe=96,rerank_width=500` balanced point:

| metric | `30184` | `30188` |
|---|---:|---:|
| recall@100 | `0.9676` | `0.9676` |
| recall mean q-time | `11.21 ms` | `10.98 ms` |
| latency p50 | `10.9 ms` | `10.7 ms` |
| latency p95 | `11.8 ms` | `11.4 ms` |
| latency p99 | `13.3 ms` | `11.9 ms` |
| explain execution | `14.143 ms` | `13.281 ms` |
| postings visited | `77760` | `77760` |
| postings scored | `3784` | `1499` |
| postings pruned by bound | `73976` | `76261` |
| rerank rows | `500` | `500` |

## Interpretation

The balanced lane shows the same pruning win as the quality lane, but with a
cleaner latency story:

- recall stayed fixed
- scan volume stayed fixed
- pruning improved materially, cutting `postings scored` from `3784` to `1499`
- explain time and latency both improved modestly

That makes the score-ranked probe-order checkpoint look more convincing on the
balanced point than on the quality point.

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

