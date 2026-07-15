---
task: 181
packet: 001-coverage-landmark-plan
role: coder
status: review-requested
head: d6a1ca4507f441467c393183dd7a32eb3e776142
date: 2026-07-15
---

# Review request: bounded-head coverage and landmark benchmark plan

Please review the proposed Task 181 definition at
`plan/tasks/181-ec-distann-head-coverage-landmarks.md` and its task-index entry.
This is a planning-only checkpoint: it changes no product or benchmark code and
claims no new measurements.

## Evidence basis and boundary

Task 180's completed screen showed that exact scoring of the existing cap-4096
head reproduced production recall, width 32-256 and seeds 32-128 were flat, and
exact cap 16384 recovered only a small nominal amount. The same-run owner scan
reached 0.9970 with the same RaBitQ graph/traversal at O(N) query cost. Task 181
therefore measures which useful entry regions are absent and whether a better
bounded landmark set or hierarchy can cover them.

Task 180 remains a NO-GO under outside review and is not reopened. Task 181 is
measurement-only; production work is isolated in conditional Task 182.

## Requested review decisions

1. Do the overlap/membership/score-gap/region diagnostics directly distinguish
   missing landmark coverage from head-search approximation?
2. Is the training/evaluation separation strong enough to prevent a
   query-trained diagnostic head from leaking held-out queries?
3. Are the fixed-cap policy families broad enough to test geometry, graph, and
   query-distribution coverage without prescribing an infeasible algorithm?
4. Is the `0.9900` hierarchy trigger a reasonable stop against another linear
   cap sweep, and are all hierarchy query-work dimensions explicitly bounded?
5. Does the residual exact-neighbor trigger preserve Task 180's attribution
   ordering instead of prematurely blaming RaBitQ?
6. Are the Phase 5 selection order and NFR-017 GO gate deterministic enough to
   hand exactly one frozen candidate to Task 182?

## Validation

- `git diff --check 5fbb37f46..d6a1ca450`: pass.
- Task 180 packets 002/003, Task 179 packet 048, FR-080/081, and
  NFR-007/017/018/019/020 exist in this checkout.
- Task numbers 181 and 182 were unclaimed in the current task index.
- No tests or benchmarks were run because this checkpoint changes planning
  Markdown only.

Please leave the outside decision under this packet's `feedback/` directory.
