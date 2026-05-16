---
id: ADR-061
title: "SPIRE IMI Reshape Deferral"
status: DEFERRED
impact: Affects Task 30 Phase 9.7 routing storage experiments
date: 2026-05-09
---
# ADR-061: SPIRE IMI Reshape Deferral

## Status

Deferred.

## Context

The IMI experiment would reshape SPIRE centroid/routing storage from a single
IVF-style partitioning surface toward an inverted multi-index layout. That is a
storage-format and routing-space A/B, not a narrow runtime knob.

The available local fixture for Phase 9.7 is real10k. Baseline packet
`review/30686-spire-phase9-quality-baseline` records the current single-IVF
SPIRE index as 8.2 MiB, about 859.3 bytes per row, with recall@10 saturated by
`nprobe=16`. At this scale, IMI is unlikely to demonstrate the storage or recall
tradeoff that motivates the larger structural change.

## Decision

Do not implement IMI reshape during Phase 9.7. Defer the treatment until a
larger local fixture can exercise the storage and routing-space tradeoff.

The current single-IVF SPIRE storage remains the Phase 9 shape. Any future IMI
implementation must be opt-in behind a reloption such as
`storage_format=imi` or an equivalent explicit index option, with the default
unchanged until a packet records positive A/B evidence.

## Revisit Conditions

Reopen this ADR when a local fixture at real50k or larger is available, or when
a storage benchmark can show that the current single-IVF layout is the dominant
limit at real10k.

Any reopening packet must record:

- load, storage, explain, latency, and recall lanes for both single-IVF and IMI
  on the same fixture;
- object bytes, routing object counts, route counts, candidate counts, and heap
  rerank rows;
- interaction with boundary replicas, top-graph routing, and global route
  budgets;
- default-off control surface and migration behavior for existing indexes.

## Consequences

- Phase 9 closes the IMI item as ADR-deferred with an explicit scale condition.
- No unsupported IMI reloption or partial storage format is introduced.
- Future IMI work must first provide a fixture where the expected benefit can
  be measured locally.
