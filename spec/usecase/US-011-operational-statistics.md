---
id: US-011
title: Operational Statistics
type: user-story
status: DRAFT
priority: P3-medium
traces:
  - StR-004
---
# US-011: Operational Statistics

**As** a platform engineer monitoring Ecaz in production,
**I want** aggregate statistics (total distance calculations, graph hops, cache hits) to be accessible via a SQL function,
**So that** I can monitor extension health without per-query EXPLAIN analysis.

## Acceptance Criteria

1. `SELECT * FROM ecaz_stats()` returns cumulative counters for the current backend
2. Counters include: total distance calculations, total graph hops, total linear scan pages read, quantizer cache hits/misses
3. `SELECT pg_stat_reset_shared('ecaz')` resets the counters
4. The statistics survive across queries within a session but reset on backend restart

Current staged behavior:

- On PG18, `ecaz_stats()` is live.
- Shared pgstat activation still requires `shared_preload_libraries = 'ecaz'` plus restart.
- The custom-kind reset surface remains blocked in the local PG18 tree, so `pg_stat_reset_shared('ecaz')`
  is still a documented follow-up rather than a live guarantee.
