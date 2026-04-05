---
id: ADR-011
title: "Hold planner selection off until ordered tqhnsw scan semantics are credible"
status: DECIDED
impact: HIGH for FR-009, FR-006
date: 2026-04-05
---
# ADR-011: Hold planner selection off until ordered tqhnsw scan semantics are credible

## Context

The access method now has scan descriptor lifecycle support, query validation, scan-owned prepared
query state, and a bootstrap linear non-empty scan.

It does not yet have:

- HNSW graph traversal
- `ef_search`
- distance-ordered result production
- planner-visible ordered scan semantics that match the operator class contract

Allowing the planner to pick `tqhnsw` before those semantics exist would expose incomplete query
behavior as if it were a finished ANN index.

## Decision

`amcostestimate` SHALL deliberately return prohibitive costs until ordered scan execution is
credible.

Specifically:

- `startup_cost` and `total_cost` are set to effectively maximum values
- selectivity and correlation remain non-competitive
- the planner therefore avoids choosing `tqhnsw` scans in normal execution

This is an intentional temporary gate, not the final costing model.

## Consequences

### Benefits

- Incomplete bootstrap scan behavior is not surfaced as a production planner path.
- Incremental scan execution work can land and be tested safely behind the planner gate.

### Tradeoffs

- FR-006 and FR-009 acceptance criteria that depend on planner selection are intentionally not yet
  satisfied.
- EXPLAIN-based index-scan expectations remain deferred until real ordered traversal exists.

## Follow-Up

Later scan work should remove this override only after:

1. greedy descent and layer-0 traversal are implemented
2. result ordering matches the operator class contract
3. `ef_search` is wired through scan execution
4. scan behavior is validated against brute-force reference queries
