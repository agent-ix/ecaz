# Task 123 Phase B Follow-up: 100k nlists=1024 Boundary=2

This packet continues after packet 004 rather than waiting for reviewer timing,
per operator direction. Packet 004 tested the reviewer-requested 100k
`nlists=1024` spot-check for `boundary_replica_count in {0,1}`. This packet
adds the obvious missing boundary-2 cell at the same `nlists=1024` setting and
extends the nprobe sweep to `8 / 16 / 32 / 64`.

## Request

Please review this packet together with packet 004 and confirm that the added
boundary-2 evidence does not overturn the Task 123 no-go / re-scope conclusion.

The short version: b2 improves recall, but not enough. The best b2 row reaches
only `309/320 = 0.9656` recall at nprobe 64, with clean p50 `526.0 ms`,
pipeline p50 `576.094 ms`, and a `246.0 MiB` SPIRE index.

## Evidence

- `artifacts/manifest.md`: packet-local artifact index, commands, and key result tables.
- `artifacts/task123-phase-b-100k-n1024-b2-followup-suite.json`: checked-in suite config.
- `artifacts/suite-manifest.json`: structured suite manifest.
- `artifacts/suite-results.jsonl`: normalized suite results.
- `artifacts/stage-containment-100k-n1024-b2-nprobe-8-16-32-64.jsonl`: per-query route and final containment.
- `artifacts/spire-pipeline-100k-n1024-b2-nprobe-8-16-32-64.log`: pipeline metrics, recall, local-store overlap, and candidate counters.

## Key Results

The same-run repeated 100k flat exact p50 was `161.1 ms`.

| Config | nprobe | Clean p50 / p95 | Route containment | Recall@10 | Candidates/query | Object bytes/query |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| n1024 b2 | 8 | 120.1 / 153.0 ms | 268 / 320 | 0.8375 | 2,528 | 2.0 MiB |
| n1024 b2 | 16 | 179.6 / 220.9 ms | 292 / 320 | 0.9125 | 4,779 | 3.7 MiB |
| n1024 b2 | 32 | 312.3 / 449.2 ms | 302 / 320 | 0.9438 | 9,194 | 7.2 MiB |
| n1024 b2 | 64 | 526.0 / 644.8 ms | 309 / 320 | 0.9656 | 18,416 | 14.3 MiB |

Storage:

| Config | SPIRE index size | All indexes | Total table |
| --- | ---: | ---: | ---: |
| n1024 b2 | 246.0 MiB | 248.3 MiB | 1.8 GiB |

## Interpretation

Boundary=2 moves recall in the expected direction but does not make
`nlists=1024` viable:

- Compared with packet 004 b1 at nprobe 32, b2 gains only four recalled truths
  (`302/320` vs `298/320`) while clean p50 rises from `236.1 ms` to `312.3 ms`
  and SPIRE index size rises from `167.9 MiB` to `246.0 MiB`.
- The highest tested b2 row, nprobe 64, still reaches only `309/320 = 0.9656`,
  below the review target of approximately 0.99 recall and already `3.26x` the
  same-run repeated flat p50.
- Route containment equals final recall in every row, so the residual loss is
  still route selection, not candidate scoring or rerank.

This reinforces the packet 004 conclusion: finer leaves are much faster than the
original `nlists=128` high-recall path, but they do not recover enough route
containment at low or moderate probes. Boundary replication can buy more recall,
but the cost curve is poor before reaching the requested high-recall region.

## Review Questions

1. Does this boundary-2 follow-up satisfy the remaining obvious Phase B
   spot-check axis for `nlists=1024`?
2. Do you agree Task 123 should remain a no-go / re-scope result rather than
   spending further time on the full `nlists x boundary` factorial?
