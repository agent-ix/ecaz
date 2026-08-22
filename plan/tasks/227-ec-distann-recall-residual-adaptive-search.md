# Task 227: ec_distann Recall Residual and Adaptive Search

Status: **proposed, gated on Task 226** (2026-08-21). Priority: P1 recall.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidates
GRAPH-10, GRAPH-17, GRAPH-18, and TRAV-16 through TRAV-19.

## Why

Task 188 review explicitly closed its missing frontier, reachability, and graph
quality work as a scope correction: those families were unrun, unassigned, and
not refuted. Head construction/selection later closed without a candidate, and
search budget remains the only mechanism proven to move end-to-end recall.

Before codec work or a larger fixed search budget, the program needs query-level
evidence separating seed failure, frontier starvation, exact-rerank containment,
graph reachability/build quality, and approximate neighbor-ordering error.

## Goal

Complete the missing recall attribution on the current production surface and,
only if a reliable confidence signal exists, advance one bounded adaptive-search
candidate that spends extra work on hard queries rather than every query.

## Entry gate

1. Task 226 has classified current-production BW8 transfer behavior.
2. Use one immutable 100k physical generation, disjoint diagnostic/evaluation
   query sets, and current conforming owner traversal.
3. Diagnostic results do not select build parameters from evaluation data.

## Scope

### P1 — Query-level residual attribution

- Compare current bounded seeds, current search, Task-226 BW8 where applicable,
  and the owner-oracle diagnostic on identical queries.
- Record truth containment after seeding, approximate frontier, expanded set,
  and exact rerank.
- Audit components, indegree, bridge structure, hard-query reachability, and
  monolithic-versus-sharded/stitch graph quality.
- Test whether frontier stability, score gaps, round convergence, or containment
  observations predict misses without using truth at runtime.

### P2 — At most one adaptive candidate

Choose one of confidence-based early termination, bounded extra rounds, a
frontier-stability/score-gap budget, or a conservative second traversal. Do not
combine a graph rebuild and runtime policy in one A/B.

### P3 — Decision

Screen at same-generation 100k. Advance only a useful candidate to the standard
10k/50k/100k matrix. Reopen codec work only if query-level same-seed evidence
isolates an actionable RaBitQ ordering margin; Task 189's unchanged exact arm
remains rejected.

## Non-goals

- Reopening unchanged head-capacity or head-selection experiments.
- Using truth labels in the production confidence decision.
- Stacking adaptive search with Task 222--225 materialization candidates.
- Broad codec or graph-format replacement without the required attribution.

## Acceptance

1. Every current-head recall miss is classified at the registered diagnostic
   boundaries, with unknowns reported rather than inferred away.
2. Graph/stitch and approximate-ordering families receive evidence-backed
   dispositions.
3. At most one adaptive candidate is screened and either stopped or confirmed
   at 10k/50k/100k with recall, latency, storage, and bounded-work evidence.
4. Task 189 remains closed/dormant unless its exact entry trigger is met.

## Required review packets

1. `reviews/task-227/001-plan/`
2. `reviews/task-227/002-query-level-attribution/`
3. `reviews/task-227/003-adaptive-candidate/`
4. `reviews/task-227/004-full-scale-decision/` (only after a useful screen)

## References

- Task 188 packet 008 and reviewer feedback
- Tasks 185, 207, 215, 219, and 226
- Task 189 entry gate
- Roadmap GRAPH-10 / GRAPH-17 / GRAPH-18 and TRAV-16..19
