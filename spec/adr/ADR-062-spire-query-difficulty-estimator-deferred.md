---
id: ADR-062
title: "SPIRE Query Difficulty Estimator Deferral"
status: DEFERRED
impact: Affects Task 30 Phase 9.7 adaptive routing research
date: 2026-05-09
---
# ADR-062: SPIRE Query Difficulty Estimator Deferral

## Status

Deferred.

## Context

The Phase 9.7 query difficulty estimator was a stretch item intended to improve
adaptive routing decisions. Phase 9.7 now has a deterministic adaptive
`nprobe` policy in packet `review/30687-spire-adaptive-nprobe`: it records
per-query `effective_nprobe` and `adaptive_nprobe_decision`, and it shows a
local real10k treatment point that preserves recall while reducing latency.

More complex learned or heuristic estimators overlap with the existing deferred
research ADRs:

- ADR-052, learned NN-routing classifier;
- ADR-053, learned routing reranker.

Those paths still have unresolved drift, retraining, artifact, and
deterministic evaluation questions.

## Decision

Do not add a separate query difficulty estimator in Phase 9.7. Treat the
deterministic adaptive `nprobe` diagnostics as the input signal for future
estimator research.

Any future estimator must be default-off and must not replace deterministic
route controls until it has a packet-local A/B showing recall improvement
without latency regression, or latency reduction without recall regression.

## Revisit Conditions

Reopen this ADR if adaptive `nprobe` diagnostics show a persistent failure mode
that a cheap estimator can address, such as:

- high false-reduction rate where adaptive `nprobe` drops recall;
- frequent kept-wide decisions with no recall benefit;
- hard-query subsets where static thresholds cannot separate easy and hard
  queries;
- larger local fixtures where query-level route difficulty varies enough to
  support a stable estimator.

Any reopening packet must cite ADR-052 and ADR-053, record the estimator inputs
and deterministic fallback behavior, and include packet-local recall/latency
A/B measurements.

## Consequences

- Phase 9 closes the query difficulty estimator as research-deferred rather
  than silently skipped.
- The adaptive `nprobe` snapshot columns become the first diagnostic dataset
  for future estimator design.
- Learned routing work remains out of the production path until the deferred
  research questions are answered.
