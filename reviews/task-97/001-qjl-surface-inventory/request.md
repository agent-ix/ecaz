# Task 97 Packet 001: QJL Surface Inventory

Task 97 now requires a Phase 0 inventory after the Task 96 clarification. This
packet audits which current TurboQuant cells actually reach gamma-aware QJL
scoring, and whether the standard 1536-dim fixtures exercise that path.

## Result

Proceed with Task 97, but do not use the standard 1536d/4-bit cells as QJL
evidence.

- `dim=1536,bits=4` is the tiled no-QJL lane by `qjl_enabled`.
- QJL is production-reachable today at canonical `bits=4` on non-1536
  dimensions.
- In-scope AMs for Task 97 are IVF, SPIRE, and HNSW.
- DiskANN TurboQuant is out of scope for Task 97 because its current
  search-code prefilter requires the no-QJL 4-bit lane and rejects QJL-active
  dimensions.

## Proposed Fixture

Use a synthetic local fixture at `dim=1024,bits=4,seed=42` for Task 97
correctness and local benchmark evidence across IVF, SPIRE, and HNSW. This is
QJL-active without adding any new TQ mode, bit width, or storage surface.

Standard 1536d cells should be reported as no-QJL/absent for Task 97 and
covered in the Task 99 complete index x quant x mode profile.

## Artifacts

- `artifacts/surface-inventory.md`
- `artifacts/manifest.md`
- `artifacts/qjl-mode-rule-audit.log`
- `artifacts/ivf-qjl-surface-audit.log`
- `artifacts/spire-qjl-surface-audit.log`
- `artifacts/hnsw-qjl-surface-audit.log`
- `artifacts/diskann-tq-surface-audit.log`
- `artifacts/standard-fixture-dimension-audit.log`

## Review Request

Please review the Task 97 Phase 0 inventory and the proposed non-tiled fixture.
If accepted, the next packet will be the Task 97 design packet for the
`qjl32` scalar reference plus SIMD strategy against IVF/SPIRE/HNSW current
QJL surfaces.
