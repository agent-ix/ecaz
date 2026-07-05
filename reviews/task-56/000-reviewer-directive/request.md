# Task 56 + Task 59 — Reviewer Directive (operator-authorized next lanes)

Status: **directive**

Operator authorized two next lanes (2026-05-24):

1. **Task 56 (SPIRE unsafe burndown)** — START NOW.
2. **Task 59 (Common parallel.rs + stream.rs)** — start AFTER Task
   56 makes meaningful progress (slice 002 wrappers landed, or Task
   56 closes — whichever comes first).

## Rationale

Per `plan/tasks/` survey:

- Task 56 (SPIRE): 194 unsafe blocks across `src/am/ec_spire/` —
  **largest remaining hardening lane** in the post-Task-50 sequence.
  Spec exists at `plan/tasks/56-spire-unsafe-burndown.md`. Same
  pattern as Tasks 55 (DiskANN) + 57 (IVF): pure Task 53 (P6) +
  Task 54 (P3) consumer migration.
- Task 59 (Common parallel/stream): 51 unsafe blocks across
  `src/am/common/{parallel,stream}.rs` — biggest unaddressed
  common-infra surface. Spec just landed at
  `plan/tasks/59-common-parallel-stream-burndown.md`. Adds typed-view
  wrappers (EcParallelCoordinatorView, EcParallelWorkerSlotsView,
  ReadStreamScope, PrefetchScope). Cross-AM consumer migrations
  (HNSW build_parallel, AM scan paths) deferred to follow-on
  tasks per §Non-Goals.

Sequenced this way because:

- Task 56 is high-impact per task slot (largest remaining lane;
  proven SPIRE-already-flagged migration pattern from Task 54/005
  handoff list).
- Task 59 adds wrappers that **Task 58 follow-up needs** for the
  remaining HNSW build_parallel lift (resolves the standing Task 58
  close-below-floor block). Best to land Task 59 wrappers AFTER
  Task 56 demonstrates the proven pattern is still working
  end-to-end post-Task-57.
- Both lanes are scope-locked to their owning subsystems — no
  cross-AM drift.

## Coder workflow expectations (post-Task-57 corrected pattern)

Task 57/005 (just closed) established the **correct** reviewer/coder
loop pattern after 5 prior close-merge-without-signoff cycles. Apply
the same pattern on Task 56 + Task 59 from slice 001 onward:

1. **Slice plan first** (`reviews/task-NN/001-execution-plan/request.md`),
   commit, wait for reviewer plan-approval before slice 002.
2. **Per-slice request.md + code** committed separately.
3. **Bench gate** must run as part of the close — no "deferred to
   operator opt-in" / "bit-for-bit identical assertion" without
   evidence. Per `feedback_no_premature_task_close` HARD RULE.
4. **Coder reply file** at close documenting disposition vs
   reviewer asks (per `2026-05-24-03-coder.md` template).
5. **Wait for reviewer signoff before merging** to main. The 5
   prior tasks that merged without signoff degraded the workflow;
   Task 57 corrected it.

## Highest-standards bar (from operator framing 2026-05-24)

Per [[feedback_dont_defer_safety_fixes]] HARD RULE: reviewer's job
is **quality control at the highest standard**. Every safety
regression / safety fix / metric-gaming `unsafe fn` / anti-pattern
A / missing `/// # Safety` doc that surfaces in review MUST BLOCK
close. None of those are legitimate deferred-follow-on category.

Coder should write code that **doesn't surface** those issues in the
first place — the Task 57 pattern (write proper safe boundaries +
typed handles + per-fn safety docs from slice 002, not retrofit
them under reviewer pressure at slice 005) is the model.

## Coordination with Task 58

Task 58 (HNSW build_parallel.rs) currently has a standing reviewer
BLOCK on its close disposition (closed at 84 above -30% floor; see
`reviews/task-58/003-closeout/feedback/2026-05-23-01-reviewer.md`).
The recommended Task 58.1 follow-up audits depend on:

- LWLock dispatch typing → naturally fits in Task 59's
  EcParallelCoordinatorView surface
- DSM accessor → ops migration → already exists in Task 52 P8
  wrappers; can be applied by a Task 58.1 packet
- Per-call `unsafe fn` delegation audit → independent of Task 59

After Task 59 closes, Task 58.1 (HNSW build_parallel follow-up to
unblock the close disposition) is the natural next lane.

## Sequence summary

```
Task 57 close-approved (this iteration; pending Debug fix + merge)
  ↓
Task 56 (SPIRE) — START NOW
  ↓ (after slice 002 wrappers land OR Task 56 closes)
Task 59 (Common parallel/stream)
  ↓
Task 58.1 (HNSW build_parallel follow-up; unblocks Task 58 close)
```

## Scope check

`git diff main...HEAD --name-only -- src/` at task open should show
only:

- For Task 56: `src/am/ec_spire/*` files
- For Task 59: `src/am/common/{parallel,stream}.rs` (plus any new
  sibling wrapper files)

Zero HNSW / IVF / DiskANN / rabitq / storage touches. Reviewer
will flag scope drift immediately per `feedback_branch_isolation`.

## Cross-references

- Task 56 spec: `plan/tasks/56-spire-unsafe-burndown.md`.
- Task 59 spec: `plan/tasks/59-common-parallel-stream-burndown.md`.
- Task 57 correct pattern: `reviews/task-57/005-closeout/feedback/`
  (5 reviewer notes; coder reply at seq 03).
- Task 55 + 57 handoff lists: `reviews/task-54/005-closeout/request.md`
  (SPIRE: 4 P3 sites + 2 P6 sites named per file:line); same shape
  for the IVF handoff was the input to Task 57.
- HARD RULES in play:
  - [[feedback_no_premature_task_close]]
  - [[feedback_dont_defer_safety_fixes]]
  - [[feedback_branch_isolation]]
  - [[feedback_view_operations_not_accessors]]
  - [[feedback_anti_pattern_b_unbounded_lifetime]]
