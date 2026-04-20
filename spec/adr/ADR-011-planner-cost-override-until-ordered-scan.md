---
id: ADR-011
title: "Hold planner selection off until ordered ec_hnsw scan semantics are credible"
status: SUPERSEDED
supersedes_notes: "Retired by D2 planner activation (2026-04-11); FR-020 cost model now wired into amcostestimate. See FR-020-cost-estimation.md for the active costing contract."
impact: HIGH for FR-009, FR-006
date: 2026-04-05
---
# ADR-011: Hold planner selection off until ordered ec_hnsw scan semantics are credible

> **SUPERSEDED (2026-04-11):** The `f64::MAX` planner override has been removed.
> `amcostestimate` now delegates to the FR-020 two-phase cost model
> (graph-traversal + linear-fallback) documented in
> [FR-020-cost-estimation.md](../functional/FR-020-cost-estimation.md).
> All four Follow-Up conditions below have been met: greedy descent + layer-0
> traversal (A3), ordered result production matching the operator class (A3),
> `ef_search` wired through scan tuning (A4, bb13a7a), and scan validation
> against brute-force (A4). The remaining PG18 `amgettreeheight` callback binding
> (FR-020-AC-4) is explicitly out of D2 scope — see FR-020-AC-4 for the
> PG18-gated follow-up. Historical context preserved below for traceability.

## Context

The access method now has scan descriptor lifecycle support, query validation, scan-owned prepared
query state, and a bootstrap linear non-empty scan.

It does not yet have:

- HNSW graph traversal
- ordered-traversal consumption of the resolved `ef_search` tuning surface
- distance-ordered result production
- planner-visible ordered scan semantics that match the operator class contract

Allowing the planner to pick `ec_hnsw` before those semantics exist would expose incomplete query
behavior as if it were a finished ANN index.

## Decision

`amcostestimate` SHALL deliberately return prohibitive costs until ordered scan execution is
credible.

Specifically:

- `startup_cost` and `total_cost` are set to effectively maximum values
- selectivity and correlation remain non-competitive
- the planner therefore avoids choosing `ec_hnsw` scans in normal execution

This is an intentional temporary gate, not the final costing model.

Planner/integration groundwork may still land behind this gate, including:

- pure cost-model helpers and unit tests that are not yet wired into `amcostestimate`
- session-level `ec_hnsw.ef_search` override registration
- relation-versus-session precedence resolution
- planner/explain/statistics-facing snapshot helpers
- planner-facing cost snapshot helpers that show modeled FR-020 outputs alongside the still-gated
  live callback contract
- planner-facing cost snapshot helpers that make current tree-height sourcing explicit, including a
  metadata-fallback seam until PG18 `amgettreeheight` wiring actually exists
- planner-facing explain snapshot helpers that report the gate state explicitly without claiming
  that EXPLAIN can yet show a ec_hnsw index scan
- planner-facing integration snapshot helpers that make the remaining runtime ordered-scan blocker
  and PG18 callback/toolchain blocker explicit without enabling planner selection

Those surfaces do not by themselves make planner-visible scans safe.

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
