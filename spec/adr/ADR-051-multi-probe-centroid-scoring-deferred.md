---
id: ADR-051
title: "Defer Standalone Multi-Probe Centroid Scoring for SPIRE"
status: DEFERRED
impact: Affects ADR-049, Task 30 Phase 9 routing-quality planning
date: 2026-05-09
---
# ADR-051: Defer Standalone Multi-Probe Centroid Scoring for SPIRE

## Status

Deferred.

This ADR records a deliberate non-goal for Task 30 Phase 9. It keeps the option
visible without adding it to the active implementation queue.

## Context

SPIRE Phase 9 is the routing-quality ladder after the Phase 8 closeout gate.
The accepted Phase 9 scope includes deeper recursion, boundary replication,
top-level graph routing, IMI reshape, adaptive `nprobe`, and anisotropic
centroid scoring.

Standalone multi-probe centroid scoring is the idea of probing additional
nearby centroid buckets by scoring alternate centroid combinations or residual
directions. It can improve IVF-style recall, but in this roadmap it overlaps
with the larger anisotropic centroid-scoring lane.

## Decision

Defer standalone multi-probe centroid scoring.

Phase 9 should spend the routing-quality budget on anisotropic centroid scoring
first. If anisotropic scoring does not cover the needed recall/QPS frontier,
multi-probe can be reopened with measurements that show a distinct benefit.

## Rationale

- Anisotropic centroid scoring is expected to subsume the biggest recall gap
  that multi-probe would target.
- Keeping both active would split the evaluation harness and make attribution
  harder: improved recall could come from scoring geometry, extra probes, or
  both.
- Phase 9 already has enough non-research implementation work in recursion,
  boundary replication, top-level graph routing, IMI, and adaptive `nprobe`.

## Reopen Criteria

Reopen this ADR only if a packet-local Phase 9 measurement shows:

- anisotropic centroid scoring has landed or been rejected;
- recall remains below target at the required QPS/storage point;
- additional multi-probe candidates improve recall independently of simply
  raising per-level `nprobe`;
- the candidate accounting can be exposed in diagnostics without obscuring the
  existing `nprobe` policy.

## Open Questions

- What diagnostic field names would distinguish "extra centroid probes" from
  relation/session/per-level `nprobe`?
- Can extra probes be bounded by query difficulty without duplicating the
  adaptive `nprobe` policy?
- Does multi-probe retain enough benefit after boundary replication and
  top-level graph routing are enabled?
