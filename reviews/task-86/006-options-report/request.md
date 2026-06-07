# Review Request: Task 86 Options Report

## Summary

This packet consolidates the Task 86 TurboVec/TurboQuant investigation and prototype results into one report.

Report: `reviews/task-86/006-options-report/artifacts/task86-options-report.md`

## Covered

- What TurboVec's TurboQuant implementation does differently from our current TQ.
- Whether the query is encoded in the same calibrated scoring space.
- Per-vector and per-query size implications.
- SIMD/kernel comparison against our current TQ paths.
- Index fit for HNSW, IVF, SPIRE, and the inspected DiskANN adapter.
- Recommended next tasks.

## Main Conclusions

- TQ+ calibration-only is the strongest candidate from the TurboVec code.
- Per-vector renorm is not justified by the normalized/IP probe.
- Byte-pair LUTs are lower priority because they lost to our existing dim-LUT scorer.
- SPIRE had a real no-format-change gap; packet 005 patches it to use the existing no-QJL 4-bit LUT path.
- The next accepted evidence should be an `ecaz bench suite` SPIRE TurboQuant lane, followed by a calibration-only TQ+ profile prototype if the suite confirms the low-risk LUT change.

## Review Focus

- Whether the report accurately summarizes packets 001 through 005.
- Whether the recommended next task order is right.
- Whether any DiskANN TurboQuant path outside the inspected adapter should be mapped before designing cross-index TQ+ metadata.
