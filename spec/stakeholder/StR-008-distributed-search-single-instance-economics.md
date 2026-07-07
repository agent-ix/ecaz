---
id: StR-008
title: Distributed Vector Search at Single-Instance Economics
type: StR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-075"
    type: "satisfied_by"
    cardinality: "1:N"
---
# StR-008: Distributed Vector Search at Single-Instance Economics

## Stakeholder Need

The ecaz platform shall serve top-k vector search over a corpus distributed
across several PostgreSQL nodes at distinct-recall and latency comparable to
the best single-instance access method, with per-query cost that does not
grow with the fraction of the corpus that must be scanned.

## Rationale

The partitioned-routing architecture (SPIRE) was measured to be unable to
deliver this. With every remediation lever applied (release-verified
substrate, epoch caching, leaf-ranking fix, SPANN closure assignment,
distance-ratio pruning, rerank economy), holding 0.99 distinct recall still
required scanning 35.7% of corpus row-instances at 50k and 78.7% at 100k —
against a ≤5% target — and the scanned fraction grows with corpus size
(evidence: `reviews/task-144/012-release-matrix-decision/` on branch
`task-144-spire-closure-ratio-pruning`; task numbers 141–146 collide across
lanes on `main`, so citations use explicit branch + packet paths). The
release single-instance anchors on the same host and corpus are IVF 100k
0.9980 distinct recall at 37.6 ms p50 and HNSW 100k 0.9795 at 20.4 ms
(`reviews/task-146/006-anchor-results/` on branch
`task-146-spire-honest-pareto-confirmation`). The root cause is
architectural: any lossy partition-level routing decision must be hedged
wider as the corpus grows. A distributed index whose per-query work is bound
by a traversal budget rather than a scanned fraction removes that failure
mode.

## Validation Criteria

This need is satisfied when a distributed access method demonstrates, on the
standard staged corpora at 10k/50k/100k with release-verified builds via
`ecaz bench suite`: distinct_recall@10 ≥ 0.999; 3-worker multinode p50 at or
below the release single-instance IVF anchor at matched recall; and a
per-query record-touch count bounded by the configured traversal budget
(beam width × hop rounds), independent of corpus size.

## Stakeholders

Primary: operators running multi-node ecaz deployments who are accountable
for query latency and recall SLOs. Secondary: the research program itself,
which requires honest, pre-registered benchmark evidence
([StR-006](./StR-006-benchmark-evidence-discipline.md)).

## Context and Assumptions

Nodes are PostgreSQL instances connected over a low-latency network
(benchmark fixture: loopback multi-instance). Corpora are rebuildable
research datasets; no backward-compatibility constraint applies to on-disk
formats. It is assumed a stable per-vector global identity exists (the
source-identity contract of ADR-068).

## Dependencies

**Upstream**: the measured SPIRE remediation verdict (branches
`task-141-spire-bench-integrity` through
`task-146-spire-honest-pareto-confirmation`). **Downstream**: the ec_distann
functional requirement family
([FR-075](../functional/index/distann/FR-075-ec-distann-access-method-surface.md)
et seq.) and gate NFRs
([NFR-017](../non-functional/NFR-017-distann-latency-recall-gate.md),
[NFR-019](../non-functional/NFR-019-distann-per-query-touch-bound.md)).

## Priority and Risk (Informative)

High: this is the successor lane to a shelved architecture; without it the
distributed story has no measured path to the recall/latency gate. Risk if
unmet: distributed deployments remain 2–10× off single-instance latency at
equal recall.
