---
task: 180
packet: 001-bounded-head-recall-plan
role: coder
status: review-requested
head: dd78b57dfd0ccae919b9af1719ccd614811890a7
date: 2026-07-14
---

# Review request: bounded-head recall attribution benchmark plan

Please review the Task 180 definition at
`plan/tasks/180-ec-distann-bounded-head-recall-attribution.md` and its task-index
entry. This is a measurement-plan checkpoint; it does not change product code
or claim new benchmark results.

## Scope and evidence basis

Task 179 is kept closed. The task starts from its accepted evidence:

- packet 048 isolated owner-scan versus persisted-head seeding at one source
  head and found 100k recall `0.9950 -> 0.9500` while both paths retained the
  same graph, search budget, and RaBitQ neighbor representation;
- packet 038 found cap 256 to 4096 recovered only `0.005` at 100k; and
- packet 066 found BW16/H25 slightly improved recall but materially regressed
  latency, so changing distributed traversal shape is not silently bundled
  into this head attribution.

The new plan separates bounded-sample coverage, approximate head-graph search,
and returned-seed width. Exact neighbor scoring is a conditional diagnostic
only after bounded seeding approaches the owner-scan oracle but still misses
NFR-017.

## Requested decisions

1. Does the Phase 1 ordering isolate one variable per A/B without repeating
   already-settled codec or traversal experiments?
2. Are the cap-growth and exact-neighbor trigger conditions sufficiently
   pre-registered to prevent post-hoc arm selection?
3. Is the Phase 2 candidate-selection order deterministic and conservative?
4. Is it correct to require the normative NFR-017 recall/latency gate before
   issuing GO for a separate production implementation task?
5. Are the boundaries between completed Task 179, measurement-only Task 180,
   and the broader Task 172 telemetry/capacity program clear?

## Validation

- `git diff --check aaa6d339f..dd78b57df`: pass.
- All cited Task 179 packet requests and NFR-017 exist in this checkout.
- No test or benchmark was run because this checkpoint changes task-planning
  Markdown only.

Please leave the outside decision under this packet's `feedback/` directory.
