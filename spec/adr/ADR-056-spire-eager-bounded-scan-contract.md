---
id: ADR-056
title: "SPIRE Eager Bounded Scan Contract"
status: ACCEPTED
impact: Affects Task 30 Phase 10 execution architecture
date: 2026-05-09
---
# ADR-056: SPIRE Eager Bounded Scan Contract

## Status

Accepted.

## Context

SPIRE currently prepares a complete ordered candidate cursor during `amrescan`
and drains that cursor from `amgettuple`. Phase 10 needs this behavior to be
intentional before adding local multi-store overlap, remote fanout, or heap
rerank batching. A streaming `amgettuple` design would need to keep snapshot,
object-store, route-frontier, delta-delete, and heap-rerank state live across
executor calls.

The current implementation already has route and candidate guardrails:

- recursive route budget: `beam_width`, `max_leaf_routes`, and
  `max_routing_expansions`;
- candidate budget: `candidate_limit = min(rerank_width, max_candidate_rows)`,
  with `rerank_width = 0` bounded by `max_candidate_rows`;
- forward-only executor semantics through `amcanbackward = false` and an
  `amgettuple` direction check.

## Decision

SPIRE remains eager for the current Phase 10 AM scan path. `amrescan` owns
snapshot loading, object-store access, route selection, candidate scoring,
candidate dedupe/truncation, and exact heap rerank. `amgettuple` only emits the
pre-ranked `(heap_tid, orderby_score)` cursor.

The memory ceiling for one scan is the bounded route frontier plus the bounded
candidate cursor, not all routed rows. The latency ceiling is still paid in
`amrescan`, so this is a bounded eager path rather than a streaming low-latency
path.

No snapshot or object-store handles are retained across `amgettuple` calls in
this contract. If a future streaming design is accepted, it needs a separate
ADR covering PostgreSQL snapshot ownership, object-store relation locks, delta
delete visibility, cancellation, and partial-cursor error behavior.

## Required Invariants

- `amrescan` refreshes root/control state and replaces all scan-local work.
- `amrescan` resolves an explicit `SpireSingleLevelScanPlan` with finite route
  and candidate budgets.
- `amgettuple` requires `amrescan` to have run first.
- `amgettuple` supports only `ForwardScanDirection`.
- `amgettuple` must not perform route expansion, object reads, delta decoding,
  or heap rerank under the eager contract.
- Scan diagnostics must expose enough plan fields and candidate counts to show
  when route or candidate budgets are limiting recall.

## Rationale

Keeping snapshot and relation/object-store ownership local to `amrescan`
matches the current pgrx callback surface and avoids leaking PostgreSQL
resource lifetimes into repeated `amgettuple` calls. It also makes the current
performance limitation explicit: the first tuple may wait for bounded route,
candidate, and rerank work to finish.

This decision does not reject streaming permanently. It defers streaming until
there is evidence that eager bounded scans are the bottleneck and until the
additional lifecycle contract is worth the complexity.

## Consequences

- Phase 10 local and remote execution work can optimize inside `amrescan`
  first: batched heap rerank, grouped local-store reads, and concurrent remote
  dispatch.
- Product latency claims must report first-tuple latency separately from cursor
  drain cost.
- A future streaming scan must not be introduced as an incremental refactor of
  the existing cursor without a new ownership and failure-mode contract.
