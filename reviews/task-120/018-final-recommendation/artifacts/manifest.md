# Artifact Manifest: Task 120 Final Recommendation

- Head SHA: `c2daca0ee736424e2a4eb5789efae9f01e6c5e0f`
- Branch: `task-120-spire-coarse-rerank-measurement-program`
- Task bucket: `reviews/task-120/018-final-recommendation`
- Created: `2026-06-22T15:19:20Z`
- Lane: synthesis / closeout recommendation
- Fixture: no new benchmark fixture; cites existing Task 120 packets
- Storage format: no new index built
- Rerank mode: no new benchmark run
- Surface: synthesis across prior isolated local/distributed packet evidence
- AWS/cloud: not used

## Purpose

This packet satisfies Task 120 acceptance criterion 7 by recording the final
promote/iterate/shelve recommendation for each SPIRE coarse-rerank location.
It is intentionally not a benchmark packet and does not authorize AWS.

## Cited Evidence

| Phase | Packet | Result used by this synthesis |
| --- | --- | --- |
| Phase 1 | `reviews/task-120/009-phase1-attribution-rerun/` | Corrected stage containment and attribution for local flat SPIRE surfaces. |
| Phase 2 | `reviews/task-120/008-phase2-rabitq-block-pruning/` | Recursive RaBitQ `l2` block pruning is not recall-safe. |
| Phase 3 | `reviews/task-120/010-phase3-budget-policy/` | Wider local exact rerank/candidate caps do not improve recall. |
| Phase 4 | `reviews/task-120/011-phase4-route-overfetch/` | Route overfetch + rowcap25k is local-only hypothesis, not default. |
| Phase 5 AWS partial | `reviews/task-120/015-phase5-aws-distributed-rerank/` | AWS 1M distributed path functions but decision-grade distributed matrix did not complete. |
| Phase 6 | `reviews/task-120/016-phase6-maintenance-fallback-invariants/` | Conservative fallback invariant recorded; no durable format/default promoted. |
| Phase 5 local multi-node | `reviews/task-120/017-phase5-local-multinode-gate/` | Local multi-node distributed gate passed at 10k/50k/100k with three worker nodes on the same host. |

## Primary Artifact

- `artifacts/task120-final-recommendation.md`

## Key Decision Lines

- Local leaf/block coarse-rerank: shelve tested `l2` cap; do not promote.
- Candidate/rerank budgets: diagnostic-only; do not promote wider exact rerank.
- Topology route refinement: iterate only; no default.
- Distributed near-data rerank: iterate only; no product claim.
- Durable summaries/sidecars/defaults: do not introduce from Task 120.

## Validation

No new test or benchmark command was run for this synthesis packet. The packet
was validated by reading the cited task packet requests, manifests, reviewer
feedback, and pushed local multi-node artifacts. All cited result lines are
from existing packet-local evidence.
