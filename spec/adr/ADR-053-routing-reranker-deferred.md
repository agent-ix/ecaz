---
id: ADR-053
title: "Defer Learned Routing Reranker for SPIRE"
status: DEFERRED
impact: Affects ADR-049, Task 30 Phase 9 routing-quality planning
date: 2026-05-09
---
# ADR-053: Defer Learned Routing Reranker for SPIRE

## Status

Deferred.

This ADR records the learned routing reranker as a future research track, not
an active Task 30 Phase 9 implementation item.

## Context

A routing reranker would take an initial set of SPIRE routing candidates and
reorder or filter them with a learned model. This is less invasive than a
direct NN-routing classifier because it can keep geometric routing as candidate
generation, but it still introduces trained artifacts and the possibility of
discarding important routes.

Phase 9 already has deterministic or lightly calibrated routing-quality work:
deeper recursion, boundary replication, top-level graph routing, IMI, adaptive
`nprobe`, and anisotropic centroid scoring. Those should establish the
non-learned baseline before adding a learned reranking layer.

## Decision

Defer the learned routing reranker.

Do not include it in Task 30 Phase 9 implementation. Keep it visible as a
future option after the deterministic routing-quality ladder has measurements.

## Rationale

- Reranking can improve candidate order, but a bad reranker can prune or
  de-prioritize the only route that contains the true neighbor.
- The cost-of-being-wrong is query-dependent and needs an evaluation harness
  before code.
- The model lifecycle has the same unresolved drift, retraining, artifact, and
  epoch-compatibility questions as the NN-routing classifier.
- A deterministic top-graph plus anisotropic scoring path is easier to debug
  and should be the Phase 9 baseline.

## Reopen Criteria

Reopen this ADR only after:

- Phase 9 deterministic routing improvements have packet-local recall/QPS
  evidence;
- reranker candidate-generation and fallback semantics are specified;
- a benchmark demonstrates a gain over adaptive `nprobe` and anisotropic
  scoring at the same candidate budget;
- diagnostics can expose reranker version, selected route count, fallback
  count, and measured false-negative risk.

## Open Questions

- Is the reranker allowed to drop routes, or only reorder them?
- How large must the initial geometric candidate pool be to make reranking safe?
- What fallback applies when reranker confidence is low or the artifact is
  missing?
- How are learned reranker artifacts tied to SPIRE epoch manifests and remote
  descriptors?
