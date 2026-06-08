# Task 85 Packet 032 Artifact Manifest

- head SHA: `d5bfafb587f08579778a61cd5f79edf4f79d0314`
- task bucket: `reviews/task-85/`
- packet path: `reviews/task-85/032-candidate-surface-stop-condition/`
- lane: evidence review / stop condition
- fixture: AWS 1M/q500 evidence inherited from Task 83, Task 84, and Task 85
- storage format: RaBitQ where SPIRE measurements apply
- rerank mode: retained `rerank_width=25` where SPIRE measurements apply
- surface isolation: packet-local evidence cited from the owning task buckets
- timestamp: `2026-06-07T15:42:17-07:00`

## Evidence Sources

| Source | Key result lines used |
| --- | --- |
| `reviews/task-83/001-target-block-rank-diagnostic/request.md` | retained surface `recall@10=0.9832`, `candidate_sum=9,213,846`; miss attribution `3` routing misses and `81` selected-leaf block-pruning/candidate-budget misses; all `81` selected-leaf misses had target block rank `>1152` |
| `reviews/task-83/002-global-cap-recovery-sweep/request.md` | cap `1280` recovered to `0.9846` only with `10,237,554` candidates; cap `1536` recovered to `0.9876` only with `12,284,852` candidates; cap `1664` recovered to `0.9892` only with `13,308,518` candidates |
| `reviews/task-84/006-closeout-no-bounded-recovery/request.md` | route prior recovered zero selected-leaf misses; k=3 at retained cap kept `recall@10=0.9832`; near-cap rescue predicates were either too weak or became blanket-cap growth |
| `reviews/task-85/004-aws-1m-block8-geometry/request.md` | block8 geometry rejected for the retained-recall latency goal |
| `reviews/task-85/005-aws-1m-per-leaf-block-cap/request.md` | per-leaf cap rejected because it changed the candidate surface and lost recall |
| `reviews/task-85/006-aws-1m-block32-geometry/request.md` | block32 geometry rejected because same-recall movement required candidate inflation |

## Decision

The candidate-surface redesign category has no remaining concrete product
lever inside Task 85. The tested mechanisms either:

- lower recall;
- require candidate growth rather than candidate reduction;
- fail to move the selected-leaf miss set; or
- require a new learned/calibrated policy contract outside the current product
  implementation evidence.

The Task 85 ledger should mark this direction `rejected`.
