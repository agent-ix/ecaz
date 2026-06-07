# Task 81 Packet 001: TurboVec TurboQuant Analysis

## Summary

This packet starts Task 81 with a source-grounded analysis of TurboVec's
TurboQuant implementation against our TurboQuant implementation only.

Key findings:

- TurboVec's implementation is a flat compressed-vector scan index, with an
  optional ID-map wrapper. It is not HNSW, DiskANN, IVF, SPIRE, or another ANN
  routing structure.
- TurboVec does not encode the query into the same packed database code format.
  It rotates the query into the same transformed coordinate space, applies the
  inverse of the database TQ+ calibration, builds per-query LUTs, then scores
  packed database codes directly.
- The most relevant candidate improvements for our TurboQuant are TQ+
  calibration, per-vector renormalization semantics, narrower per-query LUTs,
  32-vector blocked code layout, fused scoring/top-k, and multi-query fused
  scoring.
- Transferability is strongest for quantizer microbench/IVF-style contiguous
  candidate batches, and weaker for graph AMs until we account for traversal,
  candidate-surface shape, and payload layout.

## Artifacts

- `artifacts/turbovec-tq-analysis.md` - detailed report with local source
  references and candidate follow-up options.
- `artifacts/manifest.md` - source snapshot and artifact metadata.

## Validation

No tests or benchmarks were run. This packet is an analysis/report packet only;
Task 81 requires `ecaz bench suite` evidence before any prototype is accepted.
