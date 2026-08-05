# 10k diagnostic result summary

Code head: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`

Source of truth: `run/results.jsonl` and
`run/10k/distann-multinode-summary.log`.

| Arm | Recall | p50 latency | Physical storage |
| --- | ---: | ---: | ---: |
| physical, BW32/H8, persisted head | 0.9526 | 172.0 ms | 242,745,344 bytes |
| single-index control | 0.8971 membership / 0.8814 distinct | 32.4 ms | 115,687,424 bytes |

The physical topology gate passed on all three owners, with 10,000 source
rows, no non-owned rows, and no orphans. The physical recall used 50 queries
and 10,000 recall trials; latency used 2 warmups and 5 timed queries. This is
diagnostic evidence only: 50k and 100k are not measured.
