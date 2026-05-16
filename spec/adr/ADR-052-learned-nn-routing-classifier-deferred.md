---
id: ADR-052
title: "Defer Learned NN-Routing Classifier for SPIRE"
status: DEFERRED
impact: Affects ADR-049, Task 30 Phase 9 routing-quality planning
date: 2026-05-09
---
# ADR-052: Defer Learned NN-Routing Classifier for SPIRE

## Status

Deferred.

This ADR records the learned NN-routing classifier as a future research track,
not an active Task 30 Phase 9 implementation item.

## Context

A learned NN-routing classifier would predict which SPIRE routing partitions to
visit from a query vector, replacing or augmenting centroid/top-graph routing.
It is plausible, but it changes SPIRE from deterministic geometric routing to a
trained model with drift, retraining, and evaluation requirements.

Phase 9 intentionally prioritizes improvements that are mechanical against the
current storage and diagnostic model: deeper recursion, boundary replication,
top-level graph routing, IMI, adaptive `nprobe`, and anisotropic centroid
scoring. The only learned-like Phase 9 item is the query difficulty estimator,
and that is stretch work because it is adjacent to adaptive `nprobe` rather
than a replacement for routing.

## Decision

Defer the learned NN-routing classifier.

Do not implement it in Task 30 Phase 9. Revisit it only after non-learned
routing improvements have packet-local recall/QPS evidence and the project has
a durable evaluation and retraining story.

## Rationale

- Classifier drift is a product and operations problem, not only an algorithm
  problem.
- A wrong classifier can silently route away from true neighbors; the
  cost-of-being-wrong needs explicit measurement.
- Retraining cadence, artifact storage, rollback, and compatibility with epoch
  publication are unresolved.
- Phase 9 has lower-risk routing-quality work that preserves deterministic
  fallback paths.

## Reopen Criteria

Reopen this ADR only with an evaluation plan that answers:

- what corpus/query drift signal triggers retraining;
- where classifier artifacts live and how they are versioned with SPIRE epochs;
- how fallback routing is selected when confidence is low;
- how recall loss is bounded when the classifier is wrong;
- how packet-local benchmarks compare classifier routing to centroid, graph,
  IMI, adaptive `nprobe`, and anisotropic scoring baselines.

## Open Questions

- Is the classifier per-index, per-tenant, or shared across indexes?
- Does the model predict top routing PIDs directly or predict a budget/policy
  for existing geometric routing?
- How are model artifacts validated during `CREATE INDEX`, `REINDEX`, and
  remote epoch publication?
- What SQL diagnostics expose model version, confidence, fallback rate, and
  measured miss risk?
