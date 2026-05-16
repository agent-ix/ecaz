---
id: ADR-060
title: "SPIRE Anisotropic Centroid Scoring Deferral"
status: DEFERRED
impact: Affects Task 30 Phase 9.7 routing quality experiments
date: 2026-05-09
---
# ADR-060: SPIRE Anisotropic Centroid Scoring Deferral

## Status

Deferred.

## Context

Anisotropic centroid scoring is the highest-leverage remaining SPIRE quality
experiment from the Phase 9 review. The expected value comes from the ScaNN
line of work: better centroid scoring should improve recall at fixed routing
budget on dense embeddings.

The local Phase 9.7 baseline packet
`review/30686-spire-phase9-quality-baseline` shows that the checked-in real10k
fixture is already saturated for the current SPIRE path. Across
`rerank_width=0,25,50`, recall@10 is `0.9950` at `nprobe=8` and `1.0000` at
`nprobe=16,24,32` on the 100-query subset. That fixture does not leave enough
headroom to demonstrate a meaningful recall improvement from a new centroid
loss or scoring function.

## Decision

Do not implement anisotropic centroid scoring in Phase 9.7 against the current
real10k fixture. Keep the current centroid scoring path as the default and
defer the treatment until a harder local evaluation surface exists.

The deferred implementation must be default-off behind a reloption or GUC until
measurement shows a recall improvement at fixed `nprobe` and `rerank_width`
without an unacceptable latency regression.

## Revisit Conditions

Reopen this ADR when at least one of the following exists locally:

- a checked-in real50k or larger fixture where baseline recall@10 at a useful
  operating point is below roughly `0.95`;
- a packet-local hard-query subset derived from real10k where baseline recall
  drops below roughly `0.95`;
- a new benchmark lane that exposes centroid-assignment false negatives even
  when final rerank is exact.

Any reopening packet must record:

- baseline and treatment recall/latency at the same `nprobe`, `rerank_width`,
  query set, seed, and fixture;
- the scoring loss or anisotropic alpha choice;
- interactions with `nprobe_per_level`, adaptive `nprobe`, and top-graph
  routing;
- default-off control surface and diagnostics for the selected scoring mode.

## Consequences

- Phase 9 closes the anisotropic item as ADR-deferred rather than silently
  skipped.
- The canonical real10k baseline remains the reference showing why this
  treatment cannot prove value on the current fixture.
- Future quality work should build or check in a harder local fixture before
  spending implementation time on anisotropic scoring.
