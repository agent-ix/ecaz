# Task 85 Packet 025: AWS Rerank Locality Measurement

## Result

This packet measures the packet 024 rerank-locality funnel metrics on AWS
1M/q500, using the packet 023 accepted V5 selected row-segment locator plus
summary fast path surface.

The repeat run preserves the accepted recall/candidate/rerank surface:

| Run | recall@10 | candidate_sum | heap_rerank_sum | p50 | p95 | p99 | max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| packet 025 first | 0.9876 | 9,213,846 | 12,500 | 243.197 ms | 309.324 ms | 2558.404 ms | 27344.473 ms |
| packet 025 repeat | 0.9876 | 9,213,846 | 12,500 | 222.140 ms | 275.753 ms | 288.894 ms | 296.358 ms |

The first pass has the same outlier shape as packet 023's first pass and is
kept for provenance. The repeat pass is the warm comparison point.

## Rerank Locality Signal

The repeat funnel output covers `500` queries and `12,500` rerank-prefix rows
(`25` rows/query). The prefix is highly scattered across heap blocks:

| Metric | p50 | p95 | max | sum |
| --- | ---: | ---: | ---: | ---: |
| unique heap blocks/query | 22 | 25 | 25 | 10,781 |
| adjacent heap-block transitions/query | 24 | 24 | 24 | 11,732 |
| heap-block span/query | 8,366 | 8,993 | 9,235 | n/a |
| heap-block jump sum/query | 70,314 | 93,600 | 122,314 | n/a |
| max adjacent heap-block jump/query | 7,533 | 8,766 | 9,225 | n/a |

The funnel timing remains consistent with the packet 023 accepted surface:

| Funnel metric | repeat p50 | repeat p95 |
| --- | ---: | ---: |
| object-read | 26.236 ms | 27.353 ms |
| candidate-score | 57.594 ms | 59.303 ms |
| summary-score | 47.525 ms | 49.225 ms |
| row-score | 10.067 ms | 10.144 ms |

## Task 85 Ledger Decision

- Keep the packet 023 V5 plus summary fast path as the accepted same-recall
  product candidate.
- Move `candidate-set-preserving rerank locality` from `instrumenting` to
  `implementing`.
- This packet does not accept a latency win by itself. It proves the rerank
  prefix has enough heap-block scatter to justify implementing a
  candidate-set-preserving TID grouping, block-local rerank batch, or prefetch
  schedule.

## Evidence

See `artifacts/manifest.md` for command provenance and packet-local artifact
paths. AWS final status is captured as paused in
`artifacts/cloud-status-final-after-rerank-locality-q500-paused.log`.

## Next Required Work

Implement the rerank-locality lever without changing candidate selection,
rerank width, or recall semantics. The next acceptance packet must compare
against packet 023/025 warm repeat at unchanged or improved recall,
`candidate_sum`, and `heap_rerank_sum`.
