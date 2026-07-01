# Task 123 Phase B Spot-Check: 100k nlists=1024 Boundary 0/1

## Scope

This packet responds to the reviewer request in
`reviews/task-123/003-final-closeout-request/feedback/2026-06-27-01-reviewer.md`.
It runs the requested decisive 100k spot-check before closing Task 123:

- build finer `nlists=1024` SPIRE surfaces at `boundary_replica_count=0` and `1`;
- measure low nprobes `8/16/32`;
- report route containment, final recall, clean latency vs flat floor,
  candidates/query, object bytes/query, and storage.

No code changed in this slice.

## Verdict Requested

Please review this packet as the Phase B spot-check requested in packet 003.
The evidence supports closing Task 123 as a no-go / re-scope result rather than
running the full Phase B factorial.

The finer `nlists=1024` cells fix scan volume and latency, but not recall:

- `b0,np8` is fast (`75.5 ms` p50), but recall is only `223/320 = 0.6969`.
- `b1,np8` is also fast (`102.3 ms` p50), but recall is only `251/320 = 0.7844`.
- `b1,np32`, the best recall point tested, reaches only `298/320 = 0.9313`
  while clean p50 rises to `236.1 ms`, already above the repeated flat p50
  (`203.8 ms`) and still below the Phase A `nlists=128,b4,np8` recall
  (`300/320 = 0.9375`).

Route containment equals final recall in every measured row, so the deficit is
still route containment, not candidate or rerank loss.

## Evidence

Packet-local artifacts:

- `artifacts/manifest.md`
- `artifacts/task123-phase-b-100k-nlists-spotcheck-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/suite-results.jsonl`
- `artifacts/load-100k-n1024-b{0,1}-tr50-f8.log`
- `artifacts/storage-100k-n1024-b{0,1}-tr50-f8.log`
- `artifacts/latency-flat-floor-100k-repeat.log`
- `artifacts/latency-spire-100k-n1024-b{0,1}-nprobe-8-16-32.log`
- `artifacts/spire-pipeline-100k-n1024-b{0,1}-nprobe-8-16-32.log`
- `artifacts/funnel-100k-n1024-b{0,1}-nprobe-8-16-32.jsonl`
- `artifacts/stage-containment-100k-n1024-b{0,1}-nprobe-8-16-32.jsonl`

## Result Summary

The repeated 100k flat exact floor measured p50/p95 `203.8 / 425.8 ms`.

| Config | nprobe | Clean p50 / p95 | Route containment | Final recall | Candidates/query | Object bytes/query |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| n1024 b0 | 8 | 75.5 / 98.9 ms | 223 / 320 | 0.6969 | 770 | 0.6 MiB |
| n1024 b0 | 16 | 95.1 / 120.9 ms | 256 / 320 | 0.8000 | 1,458 | 1.1 MiB |
| n1024 b0 | 32 | 153.8 / 180.8 ms | 280 / 320 | 0.8750 | 2,897 | 2.3 MiB |
| n1024 b1 | 8 | 102.3 / 123.5 ms | 251 / 320 | 0.7844 | 1,619 | 1.3 MiB |
| n1024 b1 | 16 | 143.8 / 170.1 ms | 282 / 320 | 0.8812 | 3,073 | 2.4 MiB |
| n1024 b1 | 32 | 236.1 / 290.4 ms | 298 / 320 | 0.9313 | 5,984 | 4.7 MiB |

Storage:

| Config | SPIRE index size | All indexes | Total table |
| --- | ---: | ---: | ---: |
| n1024 b0 | 89.8 MiB | 92.0 MiB | 1.6 GiB |
| n1024 b1 | 167.9 MiB | 170.1 MiB | 1.7 GiB |

## Interpretation

This spot-check separates the two effects:

- Finer leaves are a real scan-volume win. Compared with Phase A 100k
  `nlists=128,b4,np8` (`31,330` candidates/query, `25.1 MiB/query`,
  `965.9 ms` p50), `n1024,b1,np8` drops to `1,619` candidates/query,
  `1.3 MiB/query`, and `102.3 ms` p50.
- Finer leaves do not recover routing precision at the low probes the reviewer
  asked to test. Even the highest tested probe, `b1,np32`, is still below the
  Phase A `n128,b4,np8` recall and far from the proposed `~0.99` viability
  target.

That makes the closeout stronger than packet 003: Phase A showed high-recall
`n128` was too slow; this Phase B spot-check shows finer `n1024` is fast but
does not recover enough recall at low probes. The remaining path is not the full
Task 123 `nlists x boundary` factorial by default; it should be re-scoped to the
IVF/SPIRE scan-efficiency and routing architecture line named in packet 003.

## Requested Reviewer Decision

Please either:

1. sign off that this spot-check satisfies the packet 003 feedback and Task 123
   can close as no-go / re-scope, or
2. name the specific additional Phase B cell that would still be decision-grade
   after `n1024,b0/b1` failed to approach high recall at low probes.
