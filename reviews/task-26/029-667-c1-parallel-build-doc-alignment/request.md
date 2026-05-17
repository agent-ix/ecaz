# Review Request: Parallel Build Documentation Alignment

## Summary

Please review commit `07e53f1`, which aligns Task 26, FR-021, and the PG18
upgrade plan with the current ADR-048 implementation state.

This is a docs-only checkpoint. It does not change runtime code or introduce
new measurements.

## Changes

- Task 26 now marks DSM pre-assembly, worker insertion, real-50k Phase 3
  measurement, and the default switch as completed/partial instead of future
  work.
- FR-021 no longer states that graph construction remains serial or that
  parallel topology must be byte-identical to serial topology.
- FR-021 now describes the current two-stage shape:
  - heap ingestion through `shm_mq` tuple streams
  - default PG18 graph assembly through ADR-048 concurrent DSM insertion
- The PG18 upgrade plan now reflects `amcanbuildparallel = true` and the
  default-on concurrent DSM graph path.

## Validation

- `rg` check for stale serial-build claims in the touched docs
- `git diff --check`

## Notes

The packet intentionally preserves the remaining `shm_mq` ingestion boundary
as current behavior. Removing it still requires a separate direct DSM ingestion
design or a narrower proof that the queue/drain path is truly superseded for a
specific build shape.
