---
id: StR-004
title: PG18 Performance and Planner Integration
type: stakeholder-requirement
status: DRAFT
derived_usecases:
  - US-006
  - US-007
  - US-008
  - US-009
  - US-010
  - US-011
---
# StR-004: PG18 Performance and Planner Integration

## Need

PostgreSQL 18 introduces async I/O (`read_stream` API), new index AM callbacks (`amgettreeheight`, `amtranslatestrategy`), custom EXPLAIN options, custom cumulative statistics, and GIN parallel build infrastructure. The current Ecaz extension targets PG17, does not use async I/O (every page read is synchronous via `ReadBufferExtended`), has a deliberately disabled cost model (`f64::MAX`), and has no parallel build or real vacuum implementation. These gaps prevent production deployment.

## Expectation

The extension SHALL:
1. Target PostgreSQL 18 as the primary platform while maintaining PG17 compatibility via feature flags
2. Integrate with the PG18 `read_stream` API for async prefetch during both graph traversal and linear scan, achieving measurable cold-cache latency reduction
3. Implement a planner-visible cost model so the query planner can choose between index scan and sequential scan
4. Report index structure height via `amgettreeheight` for planner cost refinement
5. Support parallel index build by parallelizing the heap scan and TurboQuant encoding phase
6. Implement real vacuum (soft-delete of dead heap TIDs, graph maintenance)
7. Expose per-query diagnostics via PG18 custom EXPLAIN options
8. Expose aggregate operational metrics via PG18 custom cumulative statistics
9. Register strategy translation callbacks for optimizer interoperability

## Rationale

- Thomas Munro's prototype of `read_stream` on pgvector HNSW measured **4x cold-cache speedup** — the same random-page-access pattern applies to Ecaz
- The disabled cost model means the planner **never** selects the HNSW index, forcing users to use explicit hints or GUCs — this is a production blocker
- No-op vacuum means dead tuples accumulate indefinitely, degrading scan performance and wasting storage
- Serial index build does not scale for large tables — GIN's parallel build pattern is directly applicable
- PG18's extension diagnostics APIs (EXPLAIN options, pgstat) provide the operational visibility needed for production monitoring without custom infrastructure

## Success Criteria

- Cold-cache HNSW top-10 query (50K × 1536-dim, 4-bit, m=8) latency improves by ≥ 2x when `io_method=worker` compared to synchronous I/O baseline
- Planner selects `ec_hnsw` index scan for `ORDER BY <#> LIMIT k` without manual hints
- `EXPLAIN (ecaz)` shows scan statistics (pages read, elements scored, graph expansions)
- Parallel build with 4 workers completes in ≤ 60% of serial build time for a 100K-row table
- After DELETE + VACUUM, deleted rows are not returned by subsequent scans
