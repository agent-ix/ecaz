---
id: StR-004
title: PG18 Performance and Planner Integration
type: stakeholder-requirement
status: IMPLEMENTED
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

PostgreSQL 18 is now Ecaz's primary target. The extension uses PG18-only callback and diagnostics surfaces where available while preserving PG17 compatibility behind feature flags. The remaining PG18 work is no longer basic enablement; it is measurement, tuning, and deferred scale validation.

## Expectation

The extension SHALL:
1. Target PostgreSQL 18 as the primary platform while maintaining PG17 compatibility via feature flags.
2. Use PG18 `ReadStream`, planner, EXPLAIN, statistics, and module-identity surfaces where implemented by each access method.
3. Maintain planner-visible cost models so the query planner can choose between index scan and sequential scan.
4. Report index structure height where a meaningful callback exists.
5. Support parallel HNSW index build on eligible PG18 builds, with larger-scale validation deferred to AWS/RDS-class hardware.
6. Implement live insert and vacuum behavior for the active access methods.
7. Expose per-query diagnostics via `EXPLAIN (ecaz)`.
8. Expose aggregate operational metrics via `ecaz_stats()` with shared pgstat activation when loaded through `shared_preload_libraries`.
9. Register strategy translation callbacks for optimizer interoperability.

## Rationale

- PG18's extension diagnostics APIs provide operational visibility without a custom side channel.
- Cost, ordering, and tree-height callbacks let Ecaz participate in normal planner decisions.
- ReadStream and parallel build are important for cold-cache and build-time scaling, but their product claims require controlled hardware measurements.
- Parallel index scan was investigated and is shelved because it is not the current frontier for scaling research.

## Success Criteria

- PG18 is the default build target and PG17 remains a compatibility fallback.
- Planner selection, strategy translation, and EXPLAIN diagnostics are live for the implemented access-method surfaces.
- `ecaz_stats()` is live, with shared pgstat behavior available through preload configuration.
- Parallel HNSW build has landed locally; larger-scale speedup claims are deferred to AWS/RDS-class hardware.
- Parallel index scan is marked shelved, not an active blocker.
