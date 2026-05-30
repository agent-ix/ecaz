---
id: ADR-073
title: "HNSW Staged Offline Bulk Build Direction"
status: PROPOSED
impact: Affects Task 33, FR-008, FR-021, ADR-042, ADR-048
date: 2026-05-30
---
# ADR-073: HNSW Staged Offline Bulk Build Direction

## Context

ADR-042 made Ecaz's native HNSW builder the production build path. ADR-048 then
selected concurrent DSM graph assembly as the default parallel build strategy
inside PostgreSQL.

Task 33 refreshed that decision on the local M5 host after IVF and DiskANN had
fresh M5 passes:

- `reviews/task-33/002-30213-task33-hnsw-m5-reference-refresh-run`
  measured the real50K fixture.
- `reviews/task-33/003-30214-task33-hnsw-m5-reference-refresh-100k`
  measured the locally feasible real100K fixture.

Both packets show the same shape:

- requested workers improve build wall time through the `4` worker surface;
- requested workers `8` regresses;
- recall/latency remain usable as a reference row, but HNSW is not the leading
  M5 optimization lane after IVF and DiskANN refreshes;
- the release-installed extension does not expose pgrx-test debug phase timing
  or graph-worker launch helpers, so the packets record wall time and index
  size rather than treating debug-only counters as production observability.

Task 33's stop condition says not to continue worker-threshold tuning if the M5
curve repeats the Task 26 conclusion. The real100K packet repeats that
conclusion strongly enough to choose a Phase 2 design lane.

## Decision

Move HNSW follow-up from worker-threshold tuning to a **staged/offline bulk
build lane**.

The target architecture is a two-stage build:

1. A build tool or staging subsystem constructs graph state outside the
   latency-sensitive PostgreSQL index-build callback path.
2. PostgreSQL validates the staged artifact and publishes it through the
   existing `ec_hnsw` page format and catalog-visible `CREATE INDEX`/`REINDEX`
   lifecycle.

The current ADR-048 concurrent DSM path remains the in-PostgreSQL default until
the staged path has a validated artifact contract and benchmark packets. This
ADR does not remove ADR-048.

## Requirements

A staged HNSW build design must define:

- **Artifact identity:** dimensions, distance/operator class, quantizer bits,
  seed, HNSW `m`, `ef_construction`, row count, source corpus hash, and builder
  version.
- **Row identity:** a stable mapping from staged graph node id to heap TID or a
  checked source identity that can be resolved during publish.
- **Validation:** page-format invariants, tuple count, graph bounds, neighbor
  slot bounds, entry point, layer metadata, duplicate handling, and checksum or
  hash coverage before any index becomes visible.
- **Publish lifecycle:** crash-safe staging, WAL/fsync posture, cleanup on
  failure, and `REINDEX` behavior.
- **Fallback:** the current ADR-048 in-PostgreSQL build remains available for
  small builds, unsupported artifact versions, validation failure, and normal
  live insert maintenance.
- **Quality gate:** recall must be measured against the current in-PostgreSQL
  builder on the same corpus before making staged build a recommended default.

## Non-Goals

- Replacing live insert or vacuum repair. Staged build is a bulk-build and
  `REINDEX` direction only.
- Introducing a GPU dependency into the PostgreSQL backend. GPU acceleration, if
  any, belongs to an offline tool path compatible with ADR-046's push model.
- Changing the on-disk `ec_hnsw` page format in this ADR. Any format change
  needs its own on-disk compatibility packet.
- More worker-count tuning of ADR-048. The M5 evidence has already selected the
  useful local worker surface.

## Consequences

### Positive

- Keeps PostgreSQL backend code focused on validation and publishing rather
  than long graph-construction experiments.
- Leaves ADR-048 as a stable fallback instead of complicating it further.
- Gives future CPU/GPU/off-host builders a durable artifact boundary.
- Aligns HNSW with the broader offline-artifact direction already captured by
  ADR-046.

### Negative

- Requires a new artifact contract before implementation can start.
- Creates a second bulk-build path that must be validated against the existing
  page and recall invariants.
- Does not immediately improve the current `CREATE INDEX USING ec_hnsw` path.

## Follow-Up Work

1. Draft the staged artifact schema and publish/validation lifecycle.
2. Add a narrow CPU-only prototype that emits and validates a small staged HNSW
   artifact without changing the query path.
3. Benchmark staged publish time, total build time, recall, and storage against
   ADR-048 on the same 50K/100K surfaces used by Task 33.
4. Only then decide whether the staged path should become a documented
   recommendation for larger HNSW builds.

