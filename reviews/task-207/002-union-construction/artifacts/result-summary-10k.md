# 10k partition-union A/B result summary

Code head: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`

Source of truth: `run-10k/results.jsonl` and the two per-arm
`distann-multinode-summary.log` files.

| Arm | Build shards | Recall | p50 latency | Physical storage |
| --- | ---: | ---: | ---: | ---: |
| control | 1 | 0.9529 | 188.9 ms | 242,745,344 bytes |
| candidate | 4 | 0.9615 | 185.4 ms | 244,285,440 bytes |

Both physical topology and serving gates passed on three owners with 10,000
source rows and no non-owned/orphan rows. The candidate head contained 3,729
sample entries versus 4,096 for the monolithic control because the per-shard
prefix union deduplicated overlapping partition nodes before capping. This is
10k evidence only; 50k/100k construction and full-scale decision evidence are
still open.
