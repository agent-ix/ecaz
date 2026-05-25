# Task 61 HNSW Graviton Baseline Summary

Baseline completed on `10k-medium` for 10k, 50k, and 100k DBpedia/OpenAI3 with
`ec_hnsw`, `m=16`, `ef_construction=128`, and `ef_search` sweep
`[40, 64, 100, 128, 160, 200]`.

## Headline

100k succeeded only after preserving 10k/50k artifacts and dropping their loaded
tables to recover disk. After the successful 100k run and staging cleanup, the
100 GiB data volume still had only `2.5 GiB` free, so 1M is deferred for host
capacity reasons.

## 100k Comparator

| Engine | Sweep | Recall@10 | p50 | p95 | p99 | Index |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| HNSW | ef200 | 0.9120 | 5.18 ms | 6.66 ms | 8.80 ms | 130.2 MiB |
| DiskANN Task 59 | L64 | 0.9165 | 1.81 ms | 2.36 ms | 2.56 ms | 46.1 MiB |

HNSW is slower and uses about 2.8x the index bytes at this recall neighborhood.
The current HNSW sweep also does not reach the higher DiskANN recall points, so
the next measurement should widen `ef_search` before code optimization.

## Recommendation

Do not edit HNSW code yet. First create 1M-capable disk headroom or move to a
larger Graviton profile, then run a widened 100k/1M HNSW sweep. If the widened
sweep keeps the same shape, scan-path profiling is the likely first code lane;
build time is not the 100k limiter in this packet.
