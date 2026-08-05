# 10k head-membership diagnostic

Both arms used the same 10k corpus, 3-node fixture, four build shards,
head-index cap 4096, head search width 128, 200 queries, and one benchmark
iteration. The active head contains 4096 logical IDs in each arm.

| construction | head-set overlap | membership@32 | membership@200 | end-to-end recall |
| --- | ---: | ---: | ---: | ---: |
| `stitched_bfs` | 2004/4096 = 0.4893 | 0.4319 | 0.4503 | 0.9486 |
| `partition_union` | 2004/4096 = 0.4893 | 0.4848 | 0.5389 | 0.9468 |

Head-set overlap is symmetric and uses the two captured logical-ID sets.
Membership rates are the fraction of the 200 persisted-head prediction rows
whose top-k logical IDs occur in that arm's active head. The prediction
artifacts and captured membership JSON files are the source evidence; the
values above are a compact offline summary.

The partition-union arm has higher measured head membership at both cutoffs,
but slightly lower end-to-end recall in this 10k diagnostic. This answers the
reviewer’s motivating hypothesis without supporting a construction-policy
replacement: higher head membership alone is not the limiting factor in this
run. Broader scale and selection-objective work remain separate follow-up
work, if needed.
