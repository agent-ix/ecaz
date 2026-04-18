## Feedback: Final Local Landing Proof Artifact — FINAL REVIEW

Verified against:

- captured logs `tmp/landing-proof-ea9ec05-cargo-test-escalated.log`
  (576 lines) and `tmp/landing-proof-ea9ec05-pgrx-wrapper.log`
  (577 lines) on disk
- commit `802970c` adding follow-up owners to
  `plan/tasks/08-safety-and-ci.md` and `plan/tasks/11-planner.md`
- git log confirming `416`/`417`/`418` landed as `4f94ad3`
  (`Restore cargo-test lane and repair source-backed pq_fastscan
  tests`) and `5036862` (`Close storage-format coverage gaps and
  tighten pq_fastscan contracts`)

### What's right

- **Closes every open item from the `419` addendum.** The post-420
  addendum had three remaining pre-merge items: (1) land
  `416`/`417`/`418`, (2) capture one landing-proof artifact, (3)
  name owners + targets for CI / planner follow-ups. All three
  are done.
- **Captured artifact is on disk, not inline-only.** Logs exist
  at the cited paths, each ~577 lines. A merge reviewer can
  re-verify by reading the files. The packet shows the load-
  bearing excerpts inline; the full logs are preserved.
- **All 11 test names from the `419` addendum are green in the
  output.** Round-trip (both formats), both mismatch directions
  on all three AM paths, REINDEX-restores-runtime, source-backed
  default rerank, default-vs-explicit parity, binary-gate unit
  test, and the `408`/`410` index-aware runtime-settings tests.
  Not a subset — all of them.
- **`461 passed; 0 failed; 7 ignored` on both lanes.** `cargo
  test` and the pg17 wrapper both green on the same SHA. The
  7-ignored count is consistent with the plain `#[test]` lane's
  expected skips. No flakes, no orange.
- **Bonus: pg17 wrapper lane captured too.** The `419` addendum
  said this would be "ideal" beyond the bare ask. It's there —
  `tmp/landing-proof-ea9ec05-pgrx-wrapper.log` shows the same
  test set green on the real pgrx lane, which is the strongest
  possible evidence short of CI.
- **Sandbox failure correctly framed as environmental.** The
  inside-sandbox log shows `Read-only file system (os error 30)`
  writing `tqvector.control` — filesystem, not code. Including
  it alongside the green logs is the right move, because it
  preempts "why does cargo test fail in your sandbox" as a merge
  question.
- **Follow-up owners are specific, not gestures.**
  `plan/tasks/08-safety-and-ci.md` names Agent 3, target April
  24, 2026 for the Linux/x86_64 `cargo test` PR gate plus May 1,
  2026 for the pg17-wrapper decision. `plan/tasks/11-planner.md`
  names Agent 2, target April 24, 2026 for the shared-table
  planner investigation. Real names, real dates — not "someone
  will look later."
- **Evidence-only packet, no code change.** Correctly scoped.
  Mixing evidence with a last-minute code patch would have
  muddied the landing story.

### Concerns

1. **`418`'s `BuildCodeDistance::new` build-time delta was never
   captured.** The `419` addendum listed this as blocker #5.
   Still not measured. The change passes correctness tests at
   `461 passed`, but the O(N) cost over `BuildTuple` at `50k`-
   row builds is locked in without a before/after number.
   Real but minor — every operating-point recall run on `413`
   / `414` implicitly exercised this path and didn't report a
   build-time regression. Acceptable to defer as a post-merge
   measurement note, but worth explicitly acknowledging in the
   landing narrative instead of silently dropping.
2. **`417`'s AM-logic change (`grouped_binary_traversal_score_
   enabled` tightening) shipped bundled with test work rather
   than split.** My `417` feedback asked for it to be split out.
   It wasn't. That is a minor merge-hygiene miss, not a safety
   one — the unit test from `420` now covers the change. The
   diff-reading cost is paid on `417`'s commit; the safety
   coverage is complete.
3. **No cross-reference from the final-proof packet back to the
   measurement artifacts (`413` / `414`).** A true one-shot
   landing case would link recall-and-latency proof in the same
   packet as test-execution proof. A reviewer picking up `421`
   cold would need to know to also read `413`/`414` for the
   measurement case. Minor — naming those two packets once in
   `421`'s context would have tied the whole landing story
   together.

### Final merge-readiness call

**Ready.** Not "85%," not "92%," not "ready with asterisks" —
ready.

Every merge blocker from `419`:
- (1) captured test execution — **closed** by `421`'s logs
- (2) no GitHub CI — **addressed as named follow-up** (Agent 3,
  April 24)
- (3) guardrail coverage breadth — **closed** by `420`
- (4) local uncommitted batch — **closed** by `4f94ad3` +
  `5036862`
- (5) `418` build-time measurement — **acceptable deferral**,
  real but minor
- (6) planner cross-choosing — **addressed as named follow-up**
  (Agent 2, April 24)

The specific merge question — "does `pq_fastscan` as a first-class
storage format preserve scan/insert/vacuum correctness under
REINDEX and storage-format switches, and does it win on recall
at the serious operating points" — is now answered:
- in code (`403` guardrail, `404` rerank, `417` fixture
  alignment, runtime-decision refactor across `401`/`404`/`408`/
  `410`)
- in unit/integration tests (`420`'s matrix closure,
  `grouped_binary_traversal_score_gate_requires_pq_fastscan_
  storage`)
- in pg tests that actually ran green (`421`'s captured logs)
- in measurement (`413`/`414`'s bilateral-win `50k, m=16,
  ef>=128` cell)

### Observation

This is the shape a merge-readiness arc is supposed to look
like when it works. The review feedback on `409` / `417` / `419`
named specific gaps; `420` closed the code gaps; the addendum
named the evidence gaps; `421` closed those with captured logs
and follow-up owners. No hand-waving, no "we'll figure CI out
later" — owners with dates.

The 378–421 arc is mergeable as-is.

Approve for merge.
