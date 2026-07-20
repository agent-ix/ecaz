---
id: NFR-017
title: Distann Latency and Recall Comparison Targets
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
# NFR-017: Distann Latency and Recall Comparison Targets

## Statement

Proposed target: ec_distann achieves distinct_recall@10 ≥ 0.999 at the 10k,
50k, and 100k staged real corpora with 3-worker multinode p50 latency at or
below the release single-instance IVF anchor at matched recall.

This NFR remains `PROPOSED`. On 2026-07-17 the stakeholder explicitly ruled
that `0.999` is not an enforced acceptance gate and that improvement should be
judged by the complete best-effort relative Pareto result. The numerical values
below are therefore aspirational comparison references, not release or task
thresholds. Benchmark packets should report them for context but must not use
them alone to block a demonstrably beneficial relative A/B improvement from
proceeding to production-path validation.

## Scope

- Applies to: the physically hash-sharded FR-078 multinode read path (FR-081)
  on the standard local multi-instance fixture, release-verified builds only.
- A replicated full index with serving-ownership filtering or tombstoned
  non-owner records is an optional control lane and cannot satisfy this NFR.
- Query latency includes FR-082's coordinator-local scan registration and
  release. The physical lane SHALL issue no participant pin/unpin RPC, remote
  catalog write, WAL flush, or synchronous commit per query; a lane that does
  so is non-conforming rather than a slower valid gate candidate.
- Comparison anchors: IVF 100k 0.9980 distinct recall @ 37.6 ms p50; HNSW 100k 0.9795
  @ 20.4 ms (informational) — `reviews/task-146/006-anchor-results/` on
  branch `task-146-spire-honest-pareto-confirmation`, same host/corpus/query
  set. Any comparison against those anchors SHALL reuse that exact protocol.

## Rationale

These values preserve StR-008's planning aspiration: distributed search should
approach the best single-instance method at equal recall. They also preserve a
stable protocol for measuring progress. Acceptance of a concrete change is a
separate relative Pareto decision covering recall, mean and tail latency,
storage, construction cost, bounded work, topology, and correctness. The
predecessor architecture failed the aspirational comparison with all
remediation applied.

## Measurement and Evaluation

| Metric | Aspirational target | Comparison reference | Method |
|--------|---------------------|----------------------|--------|
| distinct_recall@10, 10k/50k/100k, 3-worker | 1.000 | 0.999 | `ecaz bench suite` recall steps |
| p50 latency at matched recall, 100k, 3-worker | ≤ 30 ms | 37.6 ms IVF anchor | `ecaz bench suite` latency/pipeline steps |
| p95 latency at matched recall, 100k, 3-worker | ≤ 2× p50 | 3× p50 context line | same run |

The table contains comparison references, not pass/fail thresholds while this
NFR is `PROPOSED`. Separately, the FR-078 physical-topology audit is a mandatory
measurement-validity prerequisite: exact coverage, empty owner intersections,
one record and row per `vec_id`, zero non-owner residue, and 100% pass before
recall or latency results may be interpreted.

## Verification

Pre-registered `ecaz bench suite` matrix (release-guarded step kinds) on the
Task 146 host/corpus/query protocol, producing a four-way comparison table
(ec_distann / IVF / HNSW / best-SPIRE) in the owning review packet; every
cited number traces to `results.jsonl`.

The suite SHALL invalidate all recall and latency rows when the topology audit
is absent or fails.

**Matched-recall comparison rule (pre-registered)**: when both AMs have a
measured point at or above `0.999`, compare their cheapest such operating
points. Otherwise report each AM's measured Pareto frontier, identify the
closest-recall pair and maximum-recall points, and show both recalls alongside
both p50s without interpolation. Falling short of `0.999` is reported but does
not by itself reject a beneficial relative A/B change.

The comparison packet SHALL also include one informational injected-latency run
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
- **Downstream**: program comparison verdict recorded in ADR-085 if this NFR
  is accepted as a release gate
