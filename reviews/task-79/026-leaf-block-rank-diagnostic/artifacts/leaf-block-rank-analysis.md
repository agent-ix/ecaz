# Task 79 Leaf Block Rank Analysis

- artifact: `leaf-block-rank-100k-rabitq-global384-rw0.jsonl`
- records: 2000 exact top-10 targets from 200 queries
- lane: local PG18, RaBitQ, `task79_surface_100k`, `nprobe=96`, `rerank_width=25`
- block selector: clustered block64 summaries, global cap 384, radius weight 0.0
- pipeline result: 4,764,181 candidates, p50 43.218 ms, recall@10 0.9690

## Status Counts

| status | count |
| --- | ---: |
| `block_ranked` | 1995 |
| `not_found_in_routed_leaves` | 5 |

The 384-block selector retained 1,938 of 2,000 exact top-10 targets. That exactly matches the observed 0.9690 recall, so the recall loss is explained by block selection before rerank.

## Rank Distribution

Among the 1,995 routed/ranked targets:

| metric | block rank |
| --- | ---: |
| min | 1 |
| p50 | 9 |
| p90 | 109 |
| p95 | 223 |
| p97.5 | 419 |
| p99 | 588 |
| max | 1099 |

## Alternate Cap Survival From Same Rank File

| global cap | selected exact top-10 targets | missed |
| ---: | ---: | ---: |
| 384 | 1938 | 62 |
| 400 | 1942 | 58 |
| 416 | 1944 | 56 |
| 512 | 1965 | 35 |
| 640 | 1979 | 21 |
| 768 | 1986 | 14 |
| 896 | 1989 | 11 |
| 1024 | 1994 | 6 |
| 1280 | 1995 | 5 |

The recall gate requires at most 15 misses over 2,000 exact top-10 targets. This rank file reaches that only around cap 768, which packet 025 already measured at 9,525,502 candidates and p50 56.486 ms.

## Per-Query Survival At Cap 384

| selected exact top-10 targets per query | queries |
| ---: | ---: |
| 6 | 1 |
| 7 | 2 |
| 8 | 9 |
| 9 | 34 |
| 10 | 154 |

## Deterministic Sampling Check

For the 57 routed targets with block rank above 384, target row offsets within the 64-row block were distributed across the whole block:

| offset bucket | count |
| ---: | ---: |
| 0-7 | 15 |
| 8-15 | 5 |
| 16-23 | 4 |
| 24-31 | 3 |
| 32-39 | 4 |
| 40-47 | 6 |
| 48-55 | 9 |
| 56-63 | 11 |

Only 4 of the 57 misses were near the one-row center sample position, and only 14 were near the two deterministic sample positions used by sample2. This explains why prior sampled-row packets improved recall only marginally or degraded it.

## Readout

The main failure is not routing: only 5 exact targets are outside the routed leaves. The dominant failure is that the single mean/radius block summary ranks 57 routed true-neighbor blocks below the candidate-budget cap. A small cap increase to 400 or 416 cannot close the gap; cap 768 is required by this rank distribution, but that violates the candidate and latency gates.

The next local Task 79 slice should improve information content per selected block, not widen the block cap. The evidence points toward richer per-block representatives or another selector that can move true-neighbor blocks from ranks 400-1100 into the top 384-416 candidate budget.
