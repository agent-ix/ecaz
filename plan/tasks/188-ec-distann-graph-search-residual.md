# Task 188: ec_distann Graph and Search Residual Recall

Status: **proposed, conditional on Tasks 185--186** (2026-07-19). Priority: P2
residual-recall follow-up.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
families `GRAPH-01` through `GRAPH-18` plus gateway-derived `HEAD-27` and
`HEAD-28` when Task 185 supplies evidence, and adaptive-search `TRAV-16`
through `TRAV-19`.

## Why

The owner oracle reaches 0.9970 with the current graph, BW4/H100, and RaBitQ,
so graph/search changes are not the first recall lever. They become justified
only after Tasks 185--186 establish how much of the bounded gap remains after
better entry routing. The residual must then be separated among finite BW/H,
candidate/rerank width, graph reachability, and build/stitch quality.

## Goal

Attribute the post-entry residual and select at most one graph-construction or
adaptive-search candidate. Avoid an undifferentiated parameter sweep.

## Entry gate

Tasks 185 and 186 must freeze the best bounded head and provide same-generation
diagnostics. If entry coverage still dominates, keep this task deferred. If the
head approaches the owner oracle but final recall remains deficient, run the
residual attribution. Task 185's negative gateway screen was structurally
constrained: only 127 positive picks controlled the cap and the rest was
frequency-filled from the control pool, while its basin selector measured the
head graph rather than the production traversal graph. Therefore Task 188
must not treat gateway selection or traversal-basin diversity as exhausted;
any such candidate requires a new, explicitly larger-pool/whole-cap premise.

## Phase 1: residual attribution

On a fresh 100k physical generation:

1. compare bounded head and owner-oracle seeds under identical search;
2. vary BW with H fixed and H with BW fixed as isolated controls;
3. measure candidate frontier and exact-rerank containment of truth neighbors;
4. audit components, indegree, bridge structure, and hard-query reachability;
5. distinguish monolithic graph quality from shard closure/stitch effects; and
6. report expansions, rounds, visited nodes, exact reads, build time, graph
   bytes, and remote work.

Do not use evaluation results to choose build parameters without a separate
validation slice.

## Phase 2: candidate selection

Pre-register at most one candidate from the attributed dominant family:

- search budget/frontier/rerank (`GRAPH-01`--`GRAPH-05`);
- build degree/list/alpha/stitch (`GRAPH-06`--`GRAPH-09`);
- connectivity/bridge/gateway repair (`GRAPH-10`--`GRAPH-13`, `GRAPH-16`);
- alternate/ensemble graph (`GRAPH-14`, `GRAPH-15`); or
- bounded query-difficulty adaptation (`GRAPH-17`).

Every candidate reports build, storage, query-work, remote-work, and tail costs.
Do not combine graph rebuild and adaptive query policy in one A/B.

## Confirmation and decision

Screen at 100k using checked-in `ecaz bench suite`. Only a useful isolated
candidate proceeds to 10k/50k/100k. Advance at most one deterministic bounded
candidate that improves recall without an unacceptable latency/storage/build
tradeoff and preserves topology, lifecycle, and failure semantics. Otherwise
STOP and record whether the remaining gap is irreducible under the retained
architecture or belongs to Task 189/190.

Any persisted graph, build-default, or adaptive runtime-policy winner requires
a separate production task and appropriate ADR/spec changes.

## Required review packets

1. `reviews/task-188/001-entry-and-residual-plan/`;
2. `reviews/task-188/002-search-graph-attribution/`;
3. `reviews/task-188/003-isolated-candidate/`;
4. `reviews/task-188/004-full-scale-decision/`.

## Non-goals

- Reopening unchanged width/seed tuning or random cap growth.
- Head construction/routing owned by Tasks 185--186.
- Broad codec replacement owned by Task 189.
- Payload or transport optimization owned by Tasks 184/187.
- Multiple simultaneous graph and search changes.

## References

- Tasks 180--186.
- FR-077, FR-080, FR-081, ADR-085.
- `plan/design/ec-distann-recall-latency-roadmap.md`.
