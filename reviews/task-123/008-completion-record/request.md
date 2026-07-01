# Task 123 Completion Record

## Scope

This packet records Task 123 completion after the Phase B boundary-2 follow-up.
It adds no new benchmark run and no code change; it updates canonical task
status from **Phase B boundary-2 follow-up closeout requested** to **complete -
local 100k evidence-backed no-go / re-scope result**.

Status update commit: `a3148af85 Mark task 123 complete`.

The operator directed that Task 123 is complete and that development should not
stop waiting on intermediate review timing.

## Evidence Chain

- `reviews/task-123/001-phase-a-latency-floor-decomposition/`: Phase A
  flat-floor and SPIRE decomposition gate at 10k / 50k / 100k.
- `reviews/task-123/003-final-closeout-request/feedback/2026-06-27-01-reviewer.md`:
  reviewer asked for a cheap 100k `nlists=1024` spot-check before accepting
  closeout.
- `reviews/task-123/004-phase-b-100k-nlists-spotcheck/`: boundary 0/1
  spot-check.
- `reviews/task-123/006-phase-b-100k-n1024-b2-followup/`: boundary 2 follow-up.
- `reviews/task-123/007-phase-b-b2-status-sync/`: prior status sync after b2.

## Completion Read

Task 123 is complete as a local 100k no-go / re-scope result:

- Phase A satisfies the gate and names the binding wall. High-recall SPIRE is
  outside the 5-10x flat-floor envelope at 10k / 50k / 100k, with the decisive
  100k row at `5483.0 ms` SPIRE p50 versus `223.3 ms` flat p50.
- Phase B spot-check satisfies the cheap follow-up requested by reviewer
  feedback. `nlists=1024` lowers scan cost, but boundary 0/1/2 does not recover
  enough route containment before latency/storage costs become unattractive.
- The best b2 row reaches only `309 / 320 = 0.9656` recall at nprobe 64, with
  p50 `526.0 ms` and a `246.0 MiB` SPIRE index.
- Route containment equals final recall in Phase A and every b0/b1/b2
  spot-check row, tying the recommendation to the route-stage funnel and the
  flat-floor latency comparison.

The decisive local 100k headline is flat-exact dominance. Same-run flat exact
returns recall 1.0 at `161-204 ms`; every tested SPIRE point is either much
slower at high recall or, for the best `nlists=1024` spot-check row, both slower
and less accurate (`0.9656` recall at `526.0 ms`). Chasing approximately 0.99
recall would push SPIRE farther past the flat exact envelope, so no further local
100k runs are warranted.

## Result

No local 100k SPIRE promotion candidate lands from Task 123. Phase C is skipped
because the Phase A gate and Phase B spot-check did not produce a promising
local candidate.

This does not claim SPIRE is globally dead. The no-go is scoped to the local
single-node 100k regime, where flat exact is feasible and dominates. SPIRE's
intended opportunity remains larger distributed / disk-resident regimes where
flat exact is not the comparator to beat.

Owning follow-ups:

- IVF/SPIRE scan-efficiency line: Tasks `111` and `111e`, especially
  candidate-frontier and scan-locality work.
- SPIRE distributed read/transport line:
  `121-spire-distributed-read-transport-efficiency`, where SPIRE should prove
  value in its intended distributed regime.

Async review may still audit packets 006-008, but the task is not blocked on
that timing.
