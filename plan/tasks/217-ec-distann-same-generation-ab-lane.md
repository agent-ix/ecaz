# Task 217: ec_distann Same-Generation A/B Lane

Status: **implementation complete; packet 002 review-open** (2026-08-08). Priority: **P0 — blocking infrastructure.**

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.
Origin: Task 216 carry-in (`reviews/task-216/005-isolated-candidate-correction/`).

This is a **new task**, not a reopening of Task 216. Task 216 is review-closed;
this task builds the measurement lane that 216 identified as missing and that
its surviving candidates are explicitly blocked on.

## Why

Every ec_distann owner-side A/B to date has rebuilt a **separate physical
generation per arm**. Task 216's correction packet states the consequence
plainly:

> "The two arms also rebuilt separate generations, so their physical prediction
> difference cannot support ordered-identity attribution. Future MAT-21 work is
> blocked on a build-once/swap-extension or equivalent same-generation lane."

Two failure modes follow from separate generations, and both have already
occurred in this program:

1. **Ordered-identity claims are unsupportable.** Any difference in returned
   `(distance, vec_id)` ordering could be construction variance rather than the
   candidate. Task 216 had to withdraw such a claim.
2. **Recall deltas are confounded.** Graph construction differences move recall
   independently of the change under test, so a sub-point recall movement cannot
   be attributed.

The owner-side materialization region — `remote_materialize` 25.96 ms of a
37.21 ms `custom_scan_total` at 100k — is the last dominant latency stage in the
program. It cannot be worked without this lane.

## Goal

A benchmark lane in which **one physical generation is built once and both arms
query it**, with only the extension binary (or a runtime switch) differing
between arms, and with the generation's identity attested identical across arms
in `results.jsonl`.

## Scope

- `ecaz bench suite` gains a step or option that builds a generation once, then
  runs N arms against it, swapping the installed extension between arms.
- The generation identity (epoch fingerprint / generation digest) is emitted per
  arm and asserted equal; a mismatch fails the run rather than producing a
  quiet, unusable result.
- Extension swap is explicit and attested: each arm records
  `extension_git_sha`, `extension_build_profile`, and enabled feature flags,
  reusing the Task 197 unanimous-release preflight machinery.
- Warm-cache protocol and query-set SHA-256 are held identical across arms, as
  in the Task 215 release A/B.

## Non-goals

- Any candidate optimization. This task lands the lane and proves it; the first
  consumer is Task 218.
- Changing production behavior, defaults, formats, or the read path.
- Replacing the existing separate-generation path, which remains correct for
  construction-affecting changes.

## Acceptance

1. A 100k run with an **A/A** pair — identical extension on both arms — that
   attests one generation identity and reproduces recall byte-identically.
   An A/A that does not reproduce means the lane is not yet trustworthy.
2. A 100k run with a deliberately-different extension on the candidate arm,
   showing the generation identity still matches while the arm differs.
3. `NFR-021` conformance rows present and `conforming` on every arm; `NFR-022`
   pre-registration recorded.
4. The lane is driven by a committed `SuiteConfig`, not shell glue
   (`ecaz bench suite` only, per the repo benchmark-runner rule).

## Required review packets

1. `reviews/task-217/001-lane-contract/` — pre-registration and design.
2. `reviews/task-217/002-lane-implementation/` — the suite change plus the A/A
   and A/B proof runs.

## References

- `reviews/task-216/005-isolated-candidate-correction/` (the blocking finding)
- `reviews/task-216/001-attribution/artifacts/attribution-disposition.md`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
- Task 197 release-profile preflight
