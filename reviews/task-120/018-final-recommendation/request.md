# Task 120 Final Recommendation Review

## Request

Review the final Task 120 promote/iterate/shelve recommendation packet.

This packet does not add source changes, does not run AWS, and does not claim a
SPIRE product/default decision. It closes the measurement program in the only
defensible current state: no promotion from Task 120, with distributed
near-data rerank left as an iteration candidate only if a future user explicitly
approves the remaining AWS product-claim matrix.

## Evidence

- Manifest: `artifacts/manifest.md`
- Final recommendation: `artifacts/task120-final-recommendation.md`

The synthesis cites:

- Phase 1 corrected attribution: `reviews/task-120/009-phase1-attribution-rerun/`
- Phase 2 local leaf/block pruning: `reviews/task-120/008-phase2-rabitq-block-pruning/`
- Phase 3 budget policy: `reviews/task-120/010-phase3-budget-policy/`
- Phase 4 route overfetch: `reviews/task-120/011-phase4-route-overfetch/`
- Phase 5 AWS partial: `reviews/task-120/015-phase5-aws-distributed-rerank/`
- Phase 6 invariants: `reviews/task-120/016-phase6-maintenance-fallback-invariants/`
- Phase 5 local multi-node gate: `reviews/task-120/017-phase5-local-multinode-gate/`

## Recommendation

| Location | Recommendation |
| --- | --- |
| Local leaf/block coarse-rerank | Shelve the tested `l2` per-leaf block cap; do not promote. |
| Local candidate/rerank budgets | Keep diagnostic-only; do not promote wider exact rerank or candidate caps. |
| Topology route-set refinement | Iterate only; no product default. |
| Distributed near-data rerank | Iterate only; no product claim. |
| Durable summaries/sidecars/defaults | Do not introduce from Task 120. |

## Reviewer Focus

- Confirm the final recommendation faithfully reflects the phase evidence.
- Confirm this packet does not overclaim AWS or product readiness.
- Confirm acceptance criterion 7 is satisfied by the per-location
  promote/iterate/shelve recommendation.

## AWS Scope

AWS remains opt-in only. This packet does not authorize, request, or imply AWS
benchmarking. If a future product claim is desired, it must be a new explicitly
approved AWS matrix.
