# Review Request: SPIRE Phase 9 Routing Plan

## Summary

Task 30 now has an explicit Phase 9 routing-quality plan and the three
reviewer-requested deferred ADRs.

Planning checkpoint: `b7cf256f` (`Open SPIRE phase 9 routing plan`)

## Scope

- Adds `## Phase 9 — SPIRE Routing Quality` to
  `plan/tasks/30-spire-ivf-foundation.md`.
- Keeps Phase 9 implementation gated on the Phase 8 controlled AWS/RDS-class
  scale measurement packet, or an explicit operator waiver.
- Carries forward the reviewer-scoped Phase 9 implementation ladder:
  recursive 4-6 level catch-up, boundary replication, top-level routing graph,
  IMI reshape, adaptive `nprobe`, anisotropic centroid scoring, and query
  difficulty estimator stretch.
- Records that per-level `nprobe` was the Phase 8 pull-forward already landed
  in packet 30656.
- Adds ADR-051, ADR-052, and ADR-053 for the three deferred research/skip
  tracks:
  - standalone multi-probe centroid scoring deferred;
  - learned NN-routing classifier deferred;
  - learned routing reranker deferred.
- Updates `spec/adr/index.md` with the new ADRs.

## Validation

- `git diff --check`

## Notes

This packet intentionally does not claim Phase 8 is complete. The Phase 8 scale
packet remains open until the controlled AWS/RDS-class measurement run is
recorded, or the operator explicitly waives that gate.
