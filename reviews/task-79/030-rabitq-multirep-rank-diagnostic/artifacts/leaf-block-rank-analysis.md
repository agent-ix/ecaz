# RaBitQ Multi-Representative Leaf Block Rank Analysis

Run shape:

- local PG18 only, no AWS
- `task79_surface_100k`, 200 queries
- RaBitQ, `leaf_block_rows=32`, V4 summaries with two representatives per block
- `nprobe=96`, `rerank_width=25`
- global block cap 768, radius weight 0.25

Pipeline result:

| candidates | p50 | p95 | recall@10 |
| ---: | ---: | ---: | ---: |
| 4,860,415 | 47.616 ms | 57.164 ms | 0.9905 |

Status over 2,000 exact top-10 targets:

| status | count |
| --- | ---: |
| `block_ranked` | 1,995 |
| `not_found_in_routed_leaves` | 5 |

Configured-cap result:

| cap | selected exact top-10 targets | missed |
| ---: | ---: | ---: |
| 768 | 1,981 | 19 |

Cap readout from the same rank file:

| cap | selected exact top-10 targets | missed |
| ---: | ---: | ---: |
| 512 | 1,959 | 41 |
| 640 | 1,974 | 26 |
| 768 | 1,981 | 19 |
| 896 | 1,987 | 13 |
| 1024 | 1,989 | 11 |
| 1280 | 1,994 | 6 |

Rank distribution for routed/ranked exact top-10 targets:

| p50 | p90 | p95 | p97.5 | p99 | max |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 7 | 128 | 245 | 401 | 651 | 1309 |

Miss buckets:

| cap | bucket | count |
| ---: | --- | ---: |
| 640 | 641-768 | 7 |
| 640 | 769-896 | 6 |
| 640 | 897-1024 | 2 |
| 640 | >1024 | 6 |
| 640 | not found in routed leaves | 5 |
| 768 | 769-896 | 6 |
| 768 | 897-1024 | 2 |
| 768 | >1024 | 6 |
| 768 | not found in routed leaves | 5 |

Misses by exact truth rank at cap 768:

| truth rank | missed |
| ---: | ---: |
| 2 | 5 |
| 3 | 1 |
| 4 | 3 |
| 5 | 1 |
| 6 | 2 |
| 7 | 2 |
| 8 | 2 |
| 9 | 2 |
| 10 | 1 |

Interpretation:

- At the highest-recall capped row from packet 029, the selector is short by 4 exact top-10 targets versus the recall gate, but the row already misses the p50 gate by several milliseconds.
- At the faster cap640 row, the selector is short by 11 exact top-10 targets versus the recall gate.
- The same rank file reaches the recall gate at cap896, but block32 cap896 implies roughly 5.7M row candidates and is outside the candidate gate.
- The remaining failure is still block selection, not heap rerank width: selected exact top-10 targets match the observed recall shape.
- Five exact targets remain outside routed leaves, matching packet 026. The rest are routed but ranked too low by the current block score.
