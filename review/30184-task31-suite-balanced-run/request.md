# Task 31 Balanced Candidate Suite Run

Reviewer: please review the packet-local balanced-candidate suite execution.

## Scope

This packet runs the Task 31 balanced candidate through the new suite runner on
the already-loaded `task31_m5_real100k_pqg8_n128` surface.

The packet-local suite config covers:

- `recall100-candidates-w500`
- `latency-candidates-w500`
- `explain-balanced-candidate`

Unlike `30183`, this config drops unrelated quality thresholds because the
balanced slice does not execute the `w1000` candidate rows those thresholds
depend on.

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30184-task31-suite-balanced-run/artifacts/audit.log bench suite audit --config review/30184-task31-suite-balanced-run/task31-m5-ivf-100k-balanced.packet.json
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30184-task31-suite-balanced-run/task31-m5-ivf-100k-balanced.packet.json --manifest-output review/30184-task31-suite-balanced-run/artifacts/suite-manifest.json --results-output review/30184-task31-suite-balanced-run/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30184-task31-suite-balanced-run/artifacts/status.log bench suite status --manifest review/30184-task31-suite-balanced-run/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30184-task31-suite-balanced-run/artifacts/report.log bench suite report --manifest review/30184-task31-suite-balanced-run/artifacts/suite-manifest.json --results-output review/30184-task31-suite-balanced-run/artifacts/results.jsonl
```

No cargo or pgrx tests were run for this packet; this is a measurement-only
checkpoint.

## Results

Surface:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- balanced setting under test: `nprobe=96`, `rerank_width=500`

Recall@100:

- `nprobe=80`: `0.9639`, mean query time `10.03 ms`
- `nprobe=96`: `0.9676`, mean query time `11.21 ms`

Latency:

- `nprobe=80`: p50 `9.38 ms`, p95 `10.0 ms`, p99 `10.2 ms`
- `nprobe=96`: p50 `10.9 ms`, p95 `11.8 ms`, p99 `13.3 ms`

Representative EXPLAIN/counters for `nprobe=96,rerank_width=500`:

- index bytes: `20291584` (`19 MB`)
- execution time: `14.143 ms`
- selected lists: `96`
- posting pages read: `1815`
- postings visited: `77760`
- postings scored: `3784`
- postings pruned by bound: `73976`
- candidates inserted: `3784`
- rerank rows: `500`

## Interpretation

The balanced candidate remains the low-latency point on the `n128,p96` lane.
Compared with the threshold-passing quality packet `30183`:

- p50 drops from `12.9 ms` to `10.9 ms`
- recall@100 drops from `0.9920` to `0.9676`
- the scan volume is unchanged through selected lists, pages read, and postings
  visited, while the narrower rerank width cuts scored candidates from `6509`
  to `3784`

That keeps the existing conclusion intact: `w500` is the balanced point and
`w1000` is the quality-biased point.

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
