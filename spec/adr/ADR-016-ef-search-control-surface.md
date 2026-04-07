---
id: ADR-016
title: "Resolve ef_search through a session override over per-index defaults"
status: DECIDED
impact: HIGH for FR-009, FR-006
date: 2026-04-06
---
# ADR-016: Resolve ef_search through a session override over per-index defaults

## Context

`tqhnsw` already stores `ef_search` as an index reloption, which is the right place for a
per-index default chosen at `CREATE INDEX` time.

Planner/productization work also needs a session-level control surface:

- operators and benchmarks need a way to raise or lower search breadth per session
- planner costing should eventually consume the same resolved value as scan execution
- the planner gate in ADR-011 must remain in place while this control surface is introduced

The immediate problem is how to combine the session-level knob with the existing per-index
reloption without enabling planner-visible scans too early.

## Decision

`tqhnsw` SHALL resolve search breadth through two layers:

1. **Index reloption** `ef_search`: per-index default, stored in the relation descriptor.
2. **Session GUC** `tqhnsw.ef_search`: per-session override, registered as `PGC_USERSET`.

Precedence:

- If the session GUC is set to a non-default value, it overrides the relation reloption.
- If the session GUC remains at its default value (`40`), the relation reloption remains
  authoritative.

This means the default GUC value behaves as "no session override" rather than "force exactly 40."

The resolved setting is planner/runtime scaffolding only until ordered traversal is credible.
ADR-011 still prevents the planner from selecting `tqhnsw` scans.

## Consequences

### Benefits

- Per-index defaults continue to work without requiring users to set a session parameter.
- Benchmarks and future planner costing gain a session-level tuning surface that matches common
  PostgreSQL ANN patterns.
- The same resolution logic can be shared by planner, EXPLAIN/statistics, and eventual ordered
  traversal wiring.

### Tradeoffs

- A session cannot currently force "use the global default 40" over a non-default relation option;
  the default value means "fall back to the relation setting."
- If that distinction becomes operationally important later, the project can add an explicit
  sentinel or a second GUC for override policy.

## Follow-Up

1. Wire the resolved `ef_search` value into ordered traversal.
2. Use the same resolved value in realistic planner costing once ADR-011 is retired.
3. Extend EXPLAIN/statistics surfaces to report both the effective value and its source.
