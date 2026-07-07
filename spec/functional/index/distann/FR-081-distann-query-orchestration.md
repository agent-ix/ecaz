---
id: FR-081
title: Distann Query Orchestration and Scan Semantics
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-080"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-081: Distann Query Orchestration and Scan Semantics

## Description

A top-k scan SHALL execute as a coordinator-driven beam search: a head-index
descent followed by at most H batched hop rounds, where each round expands
the best BW unvisited frontier candidates via one parallel
`ec_distann_expand_nodes` call per owning node, merging code-scored
neighbors into the beam and exact distances into the result heap.

## Behavior

- The scan SHALL be eager (ADR-056 pattern): the orchestration loop runs at
  rescan; `amgettuple` is cursor-only over the finished result heap.
- Per hop round the coordinator SHALL: select the best BW unvisited beam
  candidates; group them by owning node
  ([FR-078](./FR-078-distann-hash-placement.md)); issue the per-node
  expansion calls in parallel over the pooled transport; merge returned
  neighbor candidates (code distances) into the beam and returned exact
  distances into the top-k heap; and mark expanded nodes visited.
- The loop SHALL terminate after H rounds, or earlier when the beam's best
  unvisited code distance cannot improve the current kth exact distance
  (convergence early-exit).
- The scan SHALL treat BW × H as the hard expansion cap
  ([NFR-019](../../../non-functional/NFR-019-distann-per-query-touch-bound.md)).
- Visited-set dedupe SHALL be by vec_id; a vec_id SHALL never be expanded
  twice in one scan.
- Final results SHALL be ordered by exact distance; no separate rerank
  round-trip is performed (exact distances arrive with expansion responses).
- The scan SHALL surface per-query counters (rounds executed, records
  expanded, candidates code-scored, per-node batch sizes, pool reuse) via
  EXPLAIN and the bench pipeline step.
- While the deployment is single-node, the same loop SHALL run with a local
  expansion function of identical signature (no transport).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-081-AC-1 | 2-node top-k results are identical to a single-node index built from the same corpus and seed | Test |
| FR-081-AC-2 | Records expanded per query ≤ BW×H in every benchmarked cell | Test (counter assertion) |
| FR-081-AC-3 | No vec_id is expanded twice within one scan | Test |
| FR-081-AC-4 | Early-exit never returns results different from running all H rounds | Test (A/B on fixed corpus) |
| FR-081-AC-5 | EXPLAIN reports the per-query traversal counters | Inspection |

## Dependencies

- **Upstream**: [FR-079](./FR-079-distann-remote-expansion-protocol.md),
  [FR-080](./FR-080-distann-coordinator-head-index.md); ADR-085 decision D9
  (termination rule)
- **Downstream**: [FR-083](./FR-083-distann-dml-path.md); the bench gate
  ([NFR-017](../../../non-functional/NFR-017-distann-latency-recall-gate.md))
