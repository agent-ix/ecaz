# Task 120 Phase 6 Maintenance and Fallback Invariants

Please review this Phase 6 invariants packet for Task 120.

## Scope

This packet records the conservative maintenance, staleness, and fallback
contract required before any SPIRE coarse summary, rerank sidecar, or query
default from Task 120 is promoted as durable behavior.

No source code changed in this packet, and no durable SPIRE format/default is
promoted here. The packet is intentionally a gate: it documents what must hold
if a later task promotes one of the measured locations.

## Result

Task 120 evidence to date does not justify promoting a durable local leaf
coarse-rerank format, a topology default, or distributed near-data rerank as a
product claim:

- Phase 2 local leaf block pruning was a measured negative result.
- Phase 3 kept candidate/rerank budget knobs diagnostic-only.
- Phase 4 carried route overfetch plus a routed-row cap forward only as an AWS
  hypothesis.
- Phase 5 has partial AWS evidence, but production-read shipping/merge metrics
  are not complete yet.

Given that state, Phase 6 records the fallback rule without creating a new
format: stale, missing, malformed, or version-skewed summaries may overfetch,
fall back to exact/full-leaf behavior, fail closed, or degraded-skip with
diagnostics, but they must not silently drop candidates.

## Evidence

- Artifact manifest:
  `reviews/task-120/016-phase6-maintenance-fallback-invariants/artifacts/manifest.md`
- Invariant record:
  `reviews/task-120/016-phase6-maintenance-fallback-invariants/artifacts/phase6-invariants.md`

## Review Notes

This is not Task 120 closeout. It satisfies the Phase 6 recordkeeping gate
before any durable format/default is promoted. The remaining closeout gap is
Phase 5 production-read shipping/merge evidence plus the final
promote/iterate/shelve packet.
