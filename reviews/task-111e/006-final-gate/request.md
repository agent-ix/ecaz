# Task 111e: Final Coarse-Rerank Gate

## Summary

This packet closes the Task 111e measurement loop for the gated
`coarse_rerank` path:

- heap-f32 `coarse_rerank` works end to end on 50k and 100k real-corpus
  fixtures;
- candidate-frontier containment is measured at 50k and 100k over
  candidate_k 25, 50, 100, 256, 512, and 1000;
- EXPLAIN exposes the coarse/frontier/rerank counters needed for review;
- the final recommendation is iterate, not promote as a default yet.

This is a review packet only. The code checkpoints for `coarse_rerank`,
contract reloptions, and SQL/admin contract tests were already committed in
earlier Task 111e packets. Packet 005 carries the compact sidecar decision:
carry table-side `f16` forward, reject `rabitq8` for the immediate high-recall
path, and keep true index-side rerank placement as follow-up scope.

## Evidence

Artifacts are under:

```text
reviews/task-111e/006-final-gate/artifacts/
```

The primary artifact index is:

```text
reviews/task-111e/006-final-gate/artifacts/manifest.md
```

Run notes:

- `suite-run.log` completed load/storage/recall/latency, then failed at the
  first EXPLAIN step because the SQL used the default index name instead of the
  actual coarse-rerank index name.
- `suite-audit-r2.log` passed after adding explicit EXPLAIN index names.
- `suite-run-explain-r2.log` completed the corrected EXPLAIN rows.
- `suite-run-frontier-r2.log` completed the 50k and 100k frontier rows.

## Key Results

### Heap-F32 Coarse-Rerank

| Corpus | Width | nprobe | Recall@10 | NDCG@10 | Latency p50 | Latency p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | 50 | 32 | 0.9940 | 0.9997 | 6.60 ms | 7.67 ms |
| 50k | 100 | 32 | 0.9960 | 0.9997 | 7.91 ms | 8.86 ms |
| 50k | 100 | 64 | 1.0000 | 1.0000 | 11.6 ms | 12.1 ms |
| 100k | 50 | 32 | 0.9710 | 0.9969 | 11.1 ms | 12.4 ms |
| 100k | 100 | 32 | 0.9730 | 0.9969 | 12.3 ms | 15.2 ms |
| 100k | 50 | 64 | 0.9980 | 1.0000 | 17.2 ms | 18.2 ms |
| 100k | 100 | 64 | 1.0000 | 1.0000 | 19.9 ms | 24.6 ms |

Index storage stayed small:

| Corpus | EC IVF index size | EC IVF bytes/row | Build index |
| --- | ---: | ---: | ---: |
| 50k | 11.6 MiB | 243.3 B | 3.61 s |
| 100k | 22.5 MiB | 235.8 B | 7.07 s |

### Candidate Frontier

The 50k frontier is viable at modest width. nprobe32/candidate_k=50 reaches
0.9940 recall at 8.572 ms total-bound p50, and candidate_k=100 only improves
to 0.9960 while rising to 10.597 ms.

The 100k frontier is the blocker. nprobe32 improves from 0.9530 at k25 to
0.9730 at k100, then stays at 0.9730 through k1000 while total-bound p50 grows
from 17.040 ms at k100 to 60.550 ms at k1000. nprobe64 reaches 0.9980 at k50
and 1.0000 at k100, but p50 rises to 25.117 ms and 27.348 ms respectively in
the frontier harness.

### EXPLAIN Coverage

Corrected EXPLAIN rows expose the required stage counters. Representative rows:

| Corpus | Width | Posting pages | Dense postings | Candidates scored | Rerank rows | Heap blocks | Approx scan | Exact rerank |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | 50 | 677 | 23,904 | 285 | 50 | 36 | 4,592 us | 1,993 us |
| 100k | 100 | 1,189 | 42,171 | 538 | 100 | 65 | 7,361 us | 3,898 us |

## Recommendation

Iterate, do not promote `coarse_rerank` as a default yet.

The path is good enough to keep behind the explicit gate: it is compact, it
works end to end, and nprobe64 can recover near-exact quality at 100k. It is
not yet a default because nprobe32 loses too much frontier recall at 100k, and
widening rerank candidates cannot recover those missing neighbors.

Recommended next levers are frontier-quality/cost improvements rather than
larger rerank width: residual or RaBitQ-2 coarse scoring, adaptive nprobe, or
better candidate bounds.

## Review Ask

Please review whether this packet, together with Task 111e packets 002 through
005, is sufficient to close Task 111e as an explicit gated implementation with
an iterate recommendation rather than a default-promotion recommendation.
