# Task 239: ec_distann Bounded-Read Overfetch

Status: **packet 001 review-closed ACCEPT — exact-main semantic harness
eager-path 12/10 reproduced; same-fixture separate-process production lazy-10
requested 6 remote and consumed 4 local for 10 returned rows (diagnostic, not a
semantic bound); packet 002 one shot consumed and failed before semantics on
the corrected CLI's 40-row versus exact-main extension's 37-row stage schema;
no rerun authorized, outside failure review pending; immediate P1 campaign
blocker before Task 229 semantic closeout**
(updated 2026-08-26). Priority: P1 correctness/performance. Decision record:
`reviews/task-239/001-current-main-reproduction/artifacts/reproduction-decision.md`;
accepted disposition:
`reviews/task-239/001-current-main-reproduction/feedback/2026-08-26-03-reviewer.md`.
Packet 002 review request:
`reviews/task-239/002-diagnosis-and-correction/request.md`.
Packet 002 seq01 verdict:
`reviews/task-239/002-diagnosis-and-correction/feedback/2026-08-26-01-reviewer.md`.
Packet 002 authorization:
`reviews/task-239/002-diagnosis-and-correction/feedback/2026-08-26-02-reviewer.md`.
Failed-run disposition:
`reviews/task-239/002-diagnosis-and-correction/artifacts/live-run-decision.md`.

Origin: Task 224 packet 003 native-control semantic evidence and reviewer
feedback `reviews/task-224/003-isolated-candidate/feedback/2026-08-25-05-reviewer.md`.
The operator's standing “GET IT DONE” campaign authorization includes creating
this bounded follow-up rather than leaving the accepted P1 divergence unnamed;
Task 224 closeout reviewer seq07 accepted that authorization record as
consistent (`reviews/task-224/003-isolated-candidate/feedback/2026-08-25-07-reviewer.md`).

## Why

Task 224's exact release 10k native control returned the correct ten rows for
`exactly_one_window`, with identical eager/lazy identity, but attributed eight
remote requests plus four locally consumed rows: twelve payload reads for ten
requested rows. Accepted Task 198 and Task 191 examples on the same scenario
reported six remote plus four local, exactly ten reads.

This is a current-path divergence with correct final results, not a MAT-26
effect. It may be a production regression, deterministic owner-split or bound
sensitivity, or an incorrect harness invariant. Blindly changing the bound
would hide which one.

## Goal

Reproduce the 12/10 result on exact current `origin/main`, distinguish runtime
regression from fixture-placement or invariant sensitivity, and either fix the
production behavior or justify a corrected bound with decision-grade evidence.

## Scope

1. Run the native, featureless/default-sender materialization semantic surface
   through a checked-in `ecaz bench suite` config. Use the same staged 10k
   corpus/query identity as Task 224 first, exact release provenance, and one
   index per table.
2. Repeat the `exactly_one_window` control under frozen generation and placement
   identity sufficiently to distinguish deterministic behavior from run or
   owner-split variation. Record requested rows, remote requests, local
   consumption, duplicate requests, returned identity, and recall.
3. Compare the current call path and bound calculation with the accepted Task
   198/191 10/10 examples. Bisect or otherwise isolate the first relevant
   behavior change if current `origin/main` reproduces 12/10.
4. If production requests redundant payloads, fix the smallest owning behavior
   and prove no weakening of lazy-10, qual deepening, mixed-owner ordering,
   restart, outage, or fail-closed semantics. If the harness bound is wrong,
   document the exact placement-sensitive invariant and evidence before changing
   it.
5. Run the complete nine-scenario native matrix and include a recall signal;
   Task 224's semantic suite skipped recall, so correctness identity alone is
   insufficient for closeout.

## Decision rules

- Do not widen `payload_reads <= requested_rows` before diagnosis.
- A runtime behavior change affecting scan, rerank, posting, payload, or storage
  requires an isolated 10k/50k/100k `ecaz bench suite` A/B with recall,
  latency, and storage evidence under the repository closeout rule.
- A harness-only invariant correction requires repeated semantic evidence,
  frozen fixture/placement provenance, recall, and a written derivation showing
  why the new bound is exact rather than permissive. Production must remain
  byte-identical.
- Task 229 may begin implementation, but neither it nor Tasks 230--233 may use
  this semantic surface as closeout evidence until Task 239 is review-closed.

## Acceptance

1. Exact current-main release evidence classifies 12/10 as a production
   regression, deterministic fixture split/bound sensitivity, or a
   non-reproducible historical observation; uncertainty is not a disposition.
2. The owning runtime behavior or invariant is corrected with independent
   evidence appropriate to the change class, without post-hoc bound widening.
3. The complete nine-scenario native matrix passes with exact result identity,
   no duplicate requests, bounded reads under the justified invariant, and a
   recorded recall result.
4. Outside review accepts the diagnosis, correction, and campaign impact; the
   task header, task index, and roadmap record the final outcome in the same
   closeout turn.

## Required review packets

1. `reviews/task-239/001-current-main-reproduction/`
2. `reviews/task-239/002-diagnosis-and-correction/`
3. `reviews/task-239/003-semantic-closeout/`
4. `reviews/task-239/004-full-scale-decision/` (only when runtime behavior
   changes and the 10k/50k/100k rule applies)

## References

- `reviews/task-224/003-isolated-candidate/artifacts/screen-decision.md`
- `reviews/task-224/003-isolated-candidate/feedback/2026-08-25-05-reviewer.md`
- Tasks 191, 198, 224, and 229
