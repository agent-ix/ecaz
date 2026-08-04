# 50k diagnostic result summary

Code head: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`

Source of truth: `run-50k-retry/results.jsonl` and
`run-50k-retry/50k/distann-multinode-summary.log`.

| Arm | Recall | p50 latency | Physical storage |
| --- | ---: | ---: | ---: |
| physical, BW32/H8, persisted head | 0.8231 | 183.5 ms | 1,242,734,592 bytes |
| single-index control | 0.7798 membership / 0.7692 distinct | 34.8 ms | 444,186,624 bytes |

The physical topology and serving gates passed on all three owners with
50,000 source rows, no non-owned rows, and no orphans. The physical recall
used 100 queries and 20,000 trials; latency used 2 warmups and 5 timed
queries. This establishes the 50k point; 100k remains open.
