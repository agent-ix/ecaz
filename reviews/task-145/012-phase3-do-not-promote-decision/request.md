# Task 145 Packet 012: Phase-3 Do-Not-Promote Decision

## Request

Please review the Task 145 Phase-3 decision packet. This is a decision-only
packet; it adds no new benchmark measurements and does not reinterpret inert
A/B results as latency or recall evidence.

## Decision

Do not promote any Task 145 rerank-economy configuration into Task 146.

The release evidence across the Task 145 packets does not show a
held-recall latency win on the transport-dominated remote path. The accepted
data points are:

- Remote rerank-width rerun: only the corrected rerun is usable. Earlier
  non-engaged or misconfigured rerank-width conclusions remain rejected.
- Remote block pruning: engaged, but negative. It exercised the mechanism and
  did not produce a latency win worth promotion.
- Large-leaf geometry: engaged and measured, but negative. It produced heavy
  block-pruning activity and modest storage reduction, but lost too much recall
  with no material latency win.
- Bound pruning: provably inert. Packet 011 supersedes packet 008 and shows
  `pre_materialization_pruned_sum=0` with `sound_bound_available_sum=0` in
  every on-arm cell.

## Evidence Used

This decision relies on the approved packet chain, not on uncited terminal
state:

- `reviews/task-145/006-remote-rerank-width-ab-rerun/`
- `reviews/task-145/007-remote-block-pruning-ab/`
- `reviews/task-145/009-large-leaf-geometry/`
- `reviews/task-145/011-remote-bound-prune-engagement-rerun/`

The packet 011 reviewer feedback is the decisive closeout for bound-prune:

```text
pre_materialization_pruned_sum = 0 on the on-arm at all three scales
sound_bound_available_sum = 0 / sound_bound_missing_sum = 43,200 in both arms
```

That makes bound-prune a provable inert/null lever in the tested remote path.
It is not an engaged negative and it is not recall-safety evidence, because the
prune branch never fired.

## Faulty Or Null Evidence Rejected

The following are not used to support this decision:

- Packet 008 latency/recall comparisons. The reviewer found the mechanism did
  not engage; packet 011 confirms that with a dedicated counter.
- Any claim that bound-prune is "recall-safe, no latency win" from packet 008.
  Recall held only vacuously because both arms ran the same effective path.
- Any conclusion based on a GUC toggle without an engagement counter proving
  that the remote worker acted on the mechanism.

The decision is therefore not "bound-prune failed after firing"; it is
"bound-prune is unusable for Task 145 promotion because it never fired under the
release remote suite."

## Scope Remaining

No Task 145 configuration should be promoted to Task 146.

If bound-prune is revisited later, it needs a runtime fix that makes the remote
worker produce sound bounds, followed by a new `ecaz bench suite` A/B where
`bound-prune-on` has `pre_materialization_pruned_sum > 0`. That is not part of
this Phase-3 closeout decision.

