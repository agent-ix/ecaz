# Task 188: ec_distann Graph and Search Residual Recall

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **complete — accept BW8 search-budget candidate; no production change**
(2026-07-26). Priority: P2 residual-recall follow-up.

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
diagnostics. Task 185's entry STOP and Task 186's packet-005 handoff satisfy the
gate, but Task 186's hierarchy result is only a query-time/arbitrary-
representative prototype STOP; it does not reject build-time hierarchy or
compressed-head alternatives. The retained entry surface is therefore the
exact-scored 16,384 head, not the hierarchy prototype.

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

The historical Phase 1 run completed only the head-vs-owner oracle, isolated
BW, and isolated H comparisons. It did not run candidate-frontier/exact-rerank
containment, graph components/indegree/bridge/hard-query reachability, or
monolithic-versus-sharded graph-quality audits. Its results therefore select a
search-budget candidate only; they do not attribute the full graph family.

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
candidate proceeds to 10k/50k/100k. Apply the decision rule at confirmation:
advance at most one deterministic bounded candidate only if the paired recall
gain has no unacceptable latency/storage/build tradeoff and topology,
lifecycle, and failure semantics hold. Otherwise STOP; do not defer the
decision to another task. Record whether the remaining gap is irreducible
under the retained architecture or belongs to Task 189/190.

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
