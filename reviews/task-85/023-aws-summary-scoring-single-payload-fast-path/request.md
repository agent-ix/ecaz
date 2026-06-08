# Task 85 Packet 023: AWS Summary Scoring Single-Payload Fast Path

## Result

This packet measures packet 022's exact-preserving single-payload summary
scoring fast path on the retained AWS 1M/q500 V5 surface. The repeat run keeps
the same recall and candidate surface while beating the retained Task 79/81
latency bar.

| Run | recall@10 | candidate_sum | heap_rerank_sum | p50 | p95 | p99 | max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| packet 023 first | 0.9876 | 9,213,846 | 12,500 | 245.446 ms | 310.607 ms | 2558.302 ms | 27342.467 ms |
| packet 023 repeat | 0.9876 | 9,213,846 | 12,500 | 222.692 ms | 275.769 ms | 286.980 ms | 296.157 ms |

The first run had a severe outlier and is retained for provenance, not for the
decision. The repeat run is the warm comparison point.

## Comparison

Packet 023 repeat beats the best retained packet 019 repeat at the same
recall/candidate/heap-rerank surface:

| Surface | recall@10 | candidate_sum | heap_rerank_sum | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| packet 019 retained repeat | 0.9876 | 9,213,846 | 12,500 | 227.388 ms | 284.166 ms | 297.164 ms |
| packet 021 V5 repeat | 0.9876 | 9,213,846 | 12,500 | 233.850 ms | 290.126 ms | 302.307 ms |
| packet 023 V5 + summary fast path repeat | 0.9876 | 9,213,846 | 12,500 | 222.692 ms | 275.769 ms | 286.980 ms |

Funnel comparison against packet 021 V5 repeat:

| Metric | packet 021 V5 repeat | packet 023 repeat |
| --- | ---: | ---: |
| object-read p50/p95 | 26.855/27.891 ms | 26.635/27.635 ms |
| candidate-score p50/p95 | 57.668/59.323 ms | 56.327/58.003 ms |
| summary-score p50/p95 | 47.597/49.224 ms | 46.270/47.924 ms |
| row-score p50/p95 | 10.067/10.173 ms | 10.063/10.168 ms |

The summary fast path is a small CPU win, but the combined V5 locator plus
summary fast path is now a real same-recall latency win versus the retained
product baseline.

## Task 85 Ledger Decision

- Accept the packet 022/023 single-payload summary-scoring fast path as part of
  the retained-recall product candidate.
- Reclassify the V5 selected row-segment locator as accepted only in
  combination with summary scoring: packet 021 alone did not beat the retained
  bar, but packet 023 proves the combined profile does.
- Keep remaining identified research directions in Task 85 scope until they
  receive packet-local accept/reject/stop evidence. They are not deferred as
  vague future work.

## Evidence

See `artifacts/manifest.md` for command provenance and packet-local artifact
paths. AWS final status is captured as paused in
`artifacts/cloud-status-final-after-summary-fast-path-q500-paused.log`.
