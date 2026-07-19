# Task 186: ec_distann Bounded Hierarchical Head

Status: **proposed, conditional on Task 185** (2026-07-19). Priority: P2 recall
capacity/routing follow-up.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`. This task
owns `HEAD-01` through `HEAD-10`, `HEAD-22` through `HEAD-24`, `HEAD-29`,
`HEAD-30`, and head-only `CODEC-08`.

## Why

Task 183 skipped trained cap growth and query-conditioned routing because its
pre-registered Phase 3 required a fixed-cap winner. That procedural skip is not
negative evidence. If Task 185 shows that a fixed 4,096 exact-scored set cannot
move the selected seed frontier sufficiently, the next question is whether a
larger stored landmark collection can improve coverage while keeping per-query
work explicitly bounded through compression or hierarchy.

## Goal

Compare a simple trained-cap capacity control with no more than two bounded
larger-head routing designs, then advance at most one benchmark candidate to a
separate production task.

## Entry gate

Task 185 must provide one of:

- a STOP showing a residual fixed-cap membership/seed limitation; or
- a winner whose remaining gap and diagnostics justify testing bounded
  capacity as an isolated follow-up.

The entry packet freezes Task 185's best policy, corpus/query identities,
selection evidence, and exact query-work accounting. Do not redesign the
fixed-cap objective inside this task.

## Candidate screen

Pre-register at most these three arms against the retained fixed-cap control:

1. trained cap 8,192 with exact scoring and 32 seeds (`HEAD-01`), solely as the
   transparent capacity/cost control;
2. one larger compressed head with a fixed approximate-score cap and bounded
   exact shortlist (`HEAD-03`, optionally head-only `CODEC-08`); and
3. one two-level representative/group design with caps on representatives,
   groups opened, landmarks scored, seeds, remote requests, cached bytes, and
   persisted bytes (`HEAD-04`, `HEAD-05`, `HEAD-07`, or `HEAD-10`).

Cap 16,384 exact scoring (`HEAD-02`) runs only if the 8,192 control shows a
useful monotonic recall signal. Head-graph navigation, ensembles, learned
routing, and multi-start remain ledger alternatives but may not be added
post-hoc to this screen.

## Correctness and evidence

- Training/validation/evaluation inputs remain disjoint.
- Every route and fallback is deterministic and capped.
- No owner scan or uncapped group opening is permitted.
- Approximate shortlist arms report same-query shortlist recall and exact seed
  identity against their full stored collection.
- Builders report peak memory, spill, time, deterministic digest, head bytes,
  cache bytes, and format implications.
- Missing/corrupt routing metadata fails closed.

Screen at 100k through checked-in `ecaz bench suite`. Only one useful isolated
candidate proceeds to 10k/50k/100k with the Task 185 measurement minimums and
complete recall/latency/storage/topology/provenance evidence.

## Decision

Advance only a candidate that improves the relative recall/latency/storage
Pareto result and has no unresolved cap, routing, fallback, format, or
construction choice. A larger stored head is acceptable only when query work
remains bounded and its cost is fully reported. Otherwise STOP.

A winner requires a separate production task and an ADR or ADR-085 amendment
when it changes persisted head format, fingerprinting, lifecycle, or upgrade
semantics.

## Required review packets

1. `reviews/task-186/001-entry-and-head-design/`;
2. `reviews/task-186/002-capacity-control/`;
3. `reviews/task-186/003-compressed-hierarchy-screen/`;
4. `reviews/task-186/004-full-scale-decision/`.

## Non-goals

- Another random-sample cap sweep.
- Unbounded owner scans or query-time work growing silently with N.
- Graph, neighbor codec, payload materialization, or DML changes.
- Production implementation or default promotion.

## References

- Tasks 180--185.
- `plan/design/ec-distann-recall-latency-roadmap.md`.
- FR-080, ADR-085, NFR-007, and NFR-017 through NFR-020.
