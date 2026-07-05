# Task 121 Status Sync: Multi-Instance Efficiency Closeout

This packet records the closeout of Task 121's reopened multi-instance
efficiency scope. It adds no new run and closes via the Task 123 negative
result.

## Standing Record (unchanged)

The original single-instance route-containment recall DOE remains **closed** and
reviewer-signed:

- `reviews/task-121/026-phase4-final-pareto-verdict/`
  (sign-off `.../feedback/2026-06-26-01-reviewer.md`).

Its recall / route-containment findings are topology-independent and retained;
no SPIRE default was promoted. `n1024 b2/tr50/f8` remains the better high-recall
follow-up candidate than `n128 b4/tr50/f8` on the local distributed path.

## What Closes

The 2026-06-27 reopen added contained multi-instance measurement for the
topology-sensitive efficiency path. That question is now resolved as a
**negative result** via Task 123:

- Recall stable (1.0000) on the contained multi-instance executor;
- communications payload bytes are not the dominant local latency driver
  (Task 123 packet 017, accepted);
- the dedupe-aware pre-materialization prune is recall-safe and latency-neutral
  but **not** a demonstrated latency win, and its leaf-side engagement was not
  measured (Task 123 packets 018/019, closeout accepted in packet 020).

Governing acceptance:
`reviews/task-123/020-post-ab-closeout-request/feedback/2026-06-30-01-reviewer.md`
Task 123 status sync: `reviews/task-123/021-post-ab-closeout/`.

The prior 2026-06-28 completion request
(`reviews/task-121/028-revised-core-algorithm-status-sync/`) and its retraction
(`reviews/task-121/029-closeout-decline-status-sync/`) are superseded by this
closeout.

## Result

Task 121 is **closed**: the original recall DOE stays closed as record, and the
reopened multi-instance efficiency question is closed via the Task 123 negative
result. No default promotion. Follow-up optimization (engagement-instrumented
prune, off-disk clean latency) moves to newer SPIRE tasks, primarily
`plan/tasks/131-spire-streaming-global-topk-pruning.md`.

## Explicit Non-Claims

No true cross-network performance, no realistic payload transport claim, no
prune latency win, no default SPIRE promotion.

## Requested Decision

Confirm Task 121 closed on the standing single-instance record plus the Task 123
multi-instance negative result; follow-up routed to Task 131. Implements the
packet 020 acceptance; no new measurement required.
