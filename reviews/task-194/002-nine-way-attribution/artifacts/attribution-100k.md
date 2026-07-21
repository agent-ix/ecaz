# Task 194 fresh 100k attribution

Fixture: three physical owners, `ec_real_100k`, 200 queries, 10 timed samples
after 3 warmups, release extension with the Task 194 measurement feature.
Topology, serving, and both remote-owner engagement checks passed before the
CLI rejected the new 28-row output schema (the stale runner expected 21).

- Recall: `0.9625`, Wilson 95% CI `0.9532–0.9700`, 2,000 trials.
- Warm latency: mean `22.0 ms`, p50 `21.3 ms`, p95 `26.4 ms`, p99 `26.9 ms`,
  max `27.0 ms`.
- Traversal total: `8.523928 ms` per scan.
- Remote expansion/transport wait: `6.870165/6.870773 ms`.
- Local expansion: `1.521645 ms`.
- Owner graph read: `0.914054 ms`; owner scoring: `0.057480 ms`.
- Coordinator partition/request encode/decode/frontier insert:
  `0.004452/0.002019/0.004298/0.022146 ms` mean stage values.
- Work: 10.3 hop rounds, 41.2 requested/returned nodes, 798.6 frontier
  insertions, and zero repeated nodes per scan.

Remote wait remains the dominant traversal component; no bounded candidate is
selected from this attribution run. The stale CLI parser failure is tooling
only—the stage rows and raw measurements were emitted successfully.
