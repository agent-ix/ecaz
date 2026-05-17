---
id: US-011
title: Operational Statistics
type: user-story
artifact_type: US
status: DRAFT
priority: P3-medium
relationships:
  - target: "ix://agent-ix/ecaz/StR-004"
    type: "derives_from"
    cardinality: "N:1"
---
# US-011: Operational Statistics

**As** a platform engineer monitoring Ecaz in production,
**I want** aggregate statistics (total distance calculations, graph hops, cache hits) to be accessible via a SQL function,
**So that** I can monitor extension health without per-query EXPLAIN analysis.

## Acceptance Criteria

### US-011-AC-1

`SELECT * FROM ecaz_stats()` returns cumulative counters for the current
backend.

### US-011-AC-2

Counters include total distance calculations, total graph hops, total linear
scan pages read, and quantizer cache hits/misses.

### US-011-AC-3

`SELECT pg_stat_reset_shared('ecaz')` resets the counters.

### US-011-AC-4

The statistics survive across queries within a session but reset on backend
restart.

Current staged behavior:

- On PG18, `ecaz_stats()` is live.
- Shared pgstat activation still requires `shared_preload_libraries = 'ecaz'` plus restart.
- The custom-kind reset surface remains blocked in the local PG18 tree, so `pg_stat_reset_shared('ecaz')`
  is still a documented follow-up rather than a live guarantee.
