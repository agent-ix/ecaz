# Task 121 Review Request: Revised Core-Algorithm Status Sync

## Scope

This packet synchronizes Task 121 with the revised Task 123 contained
multi-instance evidence. The task's original route-stage recall DOE closeout
remains intact and reviewer-signed in
`reviews/task-121/026-phase4-final-pareto-verdict/feedback/2026-06-26-01-reviewer.md`.

The reopened question was whether the named route candidates still make sense on
the real local coordinator/worker executor path. Task 123 packet
`011-multi-instance-100k-timeline-rerun` reran the two requested 100k
multi-instance cells at 200 queries.

## Core-Algorithm Result

| Config | nprobe | Queries | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| n128 b4/tr50/f8 | 8 | 200 | 662.821 ms | 923.969 ms | 0.9900 |
| n128 b4/tr50/f8 | 96 | 200 | 5408.521 ms | 5815.967 ms | 1.0000 |
| n1024 b2/tr50/f8 | 8 | 200 | 555.397 ms | 581.701 ms | 0.9290 |
| n1024 b2/tr50/f8 | 64 | 200 | 770.595 ms | 860.296 ms | 1.0000 |

This is enough for Task 121's revised core routing/recall scope: the retained
route-stage conclusions are not contradicted by the contained multi-instance
executor, and `n1024 b2/tr50/f8` remains the better high-recall follow-up
candidate than `n128 b4/tr50/f8` on the local distributed path.

## Explicit Non-Claims

This packet does not claim:

- true cross-network performance;
- realistic payload transport cost;
- complete communications attribution;
- PR #43 `ec_spire.pre_materialization_prune` A/B;
- a default SPIRE promotion.

Those are transport/materialization follow-ups. Packet 011 records the attempted
`id,source` projection failure as `remote_heap_resolution_failed`; this status
sync treats that as outside the narrowed core-algorithm completion scope.
