# Task 123 Completion Record Manifest

- Head SHA: `a3148af85669f0a5f6aa259df9a3996c06112280`
- Task bucket: `reviews/task-123/008-completion-record`
- Timestamp: `2026-06-27T17:14:49Z`
- Completion type: local 100k evidence-backed no-go / re-scope result
- Operator direction: Task 123 is complete; development is not stopped waiting
  on intermediate review timing.
- Status update commit: `a3148af85 Mark task 123 complete`

## Status Update

Files updated by the completion status commit:

- `plan/tasks/123-spire-route-precision-scan-cost.md`
- `plan/tasks/README.md`

## Evidence Sources

- `reviews/task-123/001-phase-a-latency-floor-decomposition/`: Phase A
  flat-floor and SPIRE decomposition gate at 10k / 50k / 100k.
- `reviews/task-123/003-final-closeout-request/feedback/2026-06-27-01-reviewer.md`:
  reviewer request for a cheap 100k `nlists=1024` spot-check before accepting
  closeout.
- `reviews/task-123/004-phase-b-100k-nlists-spotcheck/`: 100k
  `nlists=1024`, boundary 0/1 spot-check.
- `reviews/task-123/006-phase-b-100k-n1024-b2-followup/`: 100k
  `nlists=1024`, boundary 2 follow-up.

## Completion Findings

- Phase A high-recall SPIRE is outside the task's 5-10x flat-floor gate at
  every measured scale:
  - 10k: `496.2 ms / 29.4 ms = 16.9x`
  - 50k: `2159.5 ms / 80.2 ms = 26.9x`
  - 100k: `5483.0 ms / 223.3 ms = 24.6x`
- The 100k `nlists=1024` spot-check through boundary 2 shows finer leaves are
  faster but do not recover enough route containment:
  - b1 np32: `298 / 320 = 0.9313`
  - b2 np64: `309 / 320 = 0.9656`, p50 `526.0 ms`, SPIRE index `246.0 MiB`
- Route containment equals final recall in Phase A and every b0/b1/b2
  spot-check row, so the remaining miss is route selection and the cost verdict
  is tied to the flat-floor comparison.
- Flat exact dominates every tested local 100k SPIRE point: same-run flat exact
  returns recall 1.0 at `161-204 ms`, while the best `nlists=1024` b2 row is
  both slower and less accurate (`309 / 320 = 0.9656`, p50 `526.0 ms`).

## Result

Task 123 is complete as a local 100k no-go / re-scope result. No local 100k
SPIRE promotion candidate lands from this task. Phase C is not started because
the Phase A gate and Phase B spot-check did not produce a promising local
candidate.

The no-go is scoped to the local single-node 100k regime. SPIRE's intended
opportunity remains larger distributed / disk-resident regimes where flat exact
is not the comparator to beat.

Owning follow-ups:

- Tasks `111` and `111e` for IVF/SPIRE candidate-frontier and scan-locality work.
- `121-spire-distributed-read-transport-efficiency` for SPIRE distributed
  read/transport value in its intended regime.
