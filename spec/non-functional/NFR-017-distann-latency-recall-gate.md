---
id: NFR-017
title: Distann Latency and Recall Gate
type: NFR
status: PROPOSED
quality_attribute: performance_efficiency
relationships:
  - target: "ix://agent-ix/ecaz/StR-008"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-081"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-017: Distann Latency and Recall Gate

## Statement

The ec_distann access method SHALL achieve distinct_recall@10 ≥ 0.999 at the
10k, 50k, and 100k staged real corpora with 3-worker multinode p50 latency
at or below the release single-instance IVF anchor at matched recall.

## Scope

- Applies to: the physically hash-sharded FR-078 multinode read path (FR-081)
  on the standard local multi-instance fixture, release-verified builds only.
- A replicated full index with serving-ownership filtering or tombstoned
  non-owner records is an optional control lane and cannot satisfy this NFR.
- Anchors: IVF 100k 0.9980 distinct recall @ 37.6 ms p50; HNSW 100k 0.9795
  @ 20.4 ms (informational) — `reviews/task-146/006-anchor-results/` on
  branch `task-146-spire-honest-pareto-confirmation`, same host/corpus/query
  set. The gate comparison SHALL reuse that exact protocol.

## Rationale

This is StR-008's satisfaction bar: distributed search must not cost more
than the best single-instance method at equal recall. The predecessor
architecture failed this bar with all remediation applied.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| distinct_recall@10, 10k/50k/100k, 3-worker | 1.000 | 0.999 | `ecaz bench suite` recall steps |
| p50 latency at matched recall, 100k, 3-worker | ≤ 30 ms | ≤ 37.6 ms (IVF anchor) | `ecaz bench suite` latency/pipeline steps |
| p95 latency at matched recall, 100k, 3-worker | ≤ 2× p50 | ≤ 3× p50 | same run |
| FR-078 physical topology audit | exact coverage, empty owner intersections, one record/row per vec_id, zero non-owner residue | 100% pass before measurement | `ecaz bench suite` topology step |

## Verification

Pre-registered `ecaz bench suite` matrix (release-guarded step kinds) on the
Task 146 host/corpus/query protocol, producing a four-way comparison table
(ec_distann / IVF / HNSW / best-SPIRE) in the owning review packet; every
cited number traces to `results.jsonl`.

The suite SHALL invalidate all recall and latency rows when the topology audit
is absent or fails.

**Matched-recall comparison rule (pre-registered)**: each AM is compared at
its own cheapest operating point achieving distinct_recall@10 ≥ 0.999 on the
scale's query set; if an anchor AM has no measured point ≥ 0.999, the
comparison uses its maximum-recall measured point and reports both recalls
alongside both p50s (no interpolation).

The gate packet SHALL also include one informational injected-latency run
(ADR-085 D2: netem or equivalent per-hop delay) reporting H×RTT sensitivity;
it does not gate.

**Prerequisites**: the `distinct_recall` metric emitter (branch
`task-138-spire-distinct-recall-metric`) and the anchor evidence
(`reviews/task-146/006-anchor-results/` on branch
`task-146-spire-honest-pareto-confirmation`) must be merged to the measuring
branch before this NFR is executable; record the merge SHAs in the gate
packet manifest.

## Dependencies

- **Upstream**: [StR-008](../stakeholder/StR-008-distributed-search-single-instance-economics.md),
  [FR-078](../functional/index/distann/FR-078-distann-hash-placement.md), and
  [FR-081](../functional/index/distann/FR-081-distann-query-orchestration.md)
- **Downstream**: program-gate milestone verdict recorded in ADR-085
