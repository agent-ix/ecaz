# Task 239: ec_distann Bounded-Read Overfetch

Status: **complete — review-closed ACCEPT. Packet 001 ACCEPT reproduced the
exact-main 12/10 semantic observation; packet 002 review-closed NOT DONE after
its sole run failed pre-semantics on a 40-row/37-row cross-SHA stage schema, no
rerun; packet 003 ported the harness fix onto exact main (`21c013079` +
main-compatible config `0adea669b`) and its sole C1--C5 live run passed every
preregistered gate: all nine semantic scenarios exactly once in both logs, seven
core rows at `control_batch_size=0 candidate_batch_size=10` with exact
eager/candidate identity and zero duplicates, `exactly_one_window` at 6 remote +
4 local = 10 reads against an unchanged bound of 10, mixed/outage pass, routed
DELETE+VACUUM pass, both recall arms 0.9990 over 200 queries / 2,000 trials with
predictions byte-identical to packet 001. Disposition: HARNESS REGRESSION
CORRECTED; EXACT-MAIN LAZY-10 SEMANTIC PATH RESTORED TO 10/10. The 12/10 was a
shared-session batch-size GUC leak in the benchmark harness — a variant at batch
size 10 emitted no `SET`, so the "lazy-10" arm inherited the eager control's 0 —
not production bounded-read overfetch; the bound was never widened. `git diff
41392c011 def565270 -- src` is empty, so no production runtime behavior changed
and the 10k/50k/100k closeout matrix is NOT triggered: packet
`004-full-scale-decision` is correctly not required. No rerun was performed or is
needed; determinism rests on the same 6/4/10 split and digests at three
independent extension SHAs (task-191 `7883cfcf`, task-198 `2ff72b3e`, this run
`4ab2aa9a9`). The Task 229 / 230--233 semantic-surface blocker is LIFTED — they
may now use this surface as closeout evidence. Carried follow-ups (none owned by
Task 239, all for whichever task next touches the multinode semantic harness):
mixed/outage failures still emit no structured `pass=false` row;
`owner_payload_plan_cache` retains the same conditional-`SET` leak shape; the
recall/latency children's batch size stays inferred rather than attested.
Closeout feedback:
`reviews/task-239/003-main-baseline-semantic-proof/feedback/2026-08-26-02-reviewer.md`**
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
Packet 002 result verdict and packet 003 authorization:
`reviews/task-239/002-diagnosis-and-correction/feedback/2026-08-26-03-reviewer.md`.
Packet 003 review request:
`reviews/task-239/003-main-baseline-semantic-proof/request.md`.
Packet 003 live authorization:
`reviews/task-239/003-main-baseline-semantic-proof/feedback/2026-08-26-01-reviewer.md`.
Packet 003 result decision:
`reviews/task-239/003-main-baseline-semantic-proof/artifacts/live-run-decision.md`.
Packet 003 semantic closeout verdict (task closeout):
`reviews/task-239/003-main-baseline-semantic-proof/feedback/2026-08-26-02-reviewer.md`.
Closeout verification log:
`reviews/task-239/003-main-baseline-semantic-proof/artifacts/reviewer-seq02-verification.log`.

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
  **Lifted 2026-08-26**: Task 239 is review-closed ACCEPT, so Tasks 229 and
  230--233 may now use this semantic surface as closeout evidence
  (`reviews/task-239/003-main-baseline-semantic-proof/feedback/2026-08-26-02-reviewer.md`).

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
3. `reviews/task-239/003-semantic-closeout/` — landed as
   `reviews/task-239/003-main-baseline-semantic-proof/`
4. `reviews/task-239/004-full-scale-decision/` (only when runtime behavior
   changes and the 10k/50k/100k rule applies) — **not triggered**; the
   correction is harness-only and `git diff 41392c011 def565270 -- src` is
   empty, so no packet 004 is required

## References

- `reviews/task-224/003-isolated-candidate/artifacts/screen-decision.md`
- `reviews/task-224/003-isolated-candidate/feedback/2026-08-25-05-reviewer.md`
- Tasks 191, 198, 224, and 229
