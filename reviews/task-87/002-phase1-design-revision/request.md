# Task 87 Packet 002: Phase 1 Design Revision

## Summary

This packet responds to the Task 87/001 reviewer feedback and resolves
the four blockers before Phase 2 implementation starts.

## Changes

- HNSW batching is scoped to `TurboQuantExactScoreMode::FullLut` only.
- DiskANN grouped-PQ/RaBitQ are not reinterpreted as satisfying the
  TurboQuant no-QJL 4-bit gate.
- Follow-up Task 90 is opened for DiskANN TurboQuant search-code
  enablement or explicit Stop Condition evidence.
- HNSW will use an owned score-code scratch as the borrow source for
  `CandidateBatch`, avoiding multi-tuple lifetime coupling.
- SPIRE Phase 2 is defined as a structural batching slice; the `>= 2x`
  scoring-share gate moves to the first real batch-kernel packet.
- The RaBitQ metadata note explains why the current sketch uses an empty
  `CandidateMeta::RaBitQ` variant.

## Artifacts

- `artifacts/design-revision.md`
- `artifacts/manifest.md`

## Validation

No tests were run. This packet is documentation-only and contains no
code or behavioral change.

## Review Focus

- Confirm B1-B4 are resolved sufficiently to start Phase 2 SPIRE on this
  branch.
- Confirm Task 90 is the right follow-up scope for DiskANN TurboQuant
  search-code enablement.
- Confirm SPIRE's structural-slice gate is acceptable before a real
  batch kernel lands.
