# Task 63 HNSW RaBitQ Benchmark Handoff Head Wording

## Summary

This metadata-only packet clarifies the benchmark handoff manifest so the
faster-host instructions no longer describe a committed SHA as the "current
branch head." Later review-packet commits can advance the branch, so the
benchmark packet now records the byte-LUT-era host install SHA as a recommended
head-or-newer anchor while keeping the older scorer checkpoint as the minimum
acceptable code source.

No benchmark artifacts were changed or removed. The untracked local AMD
benchmark files under `benchmarks/task63-hnsw-rabitq-format/artifacts/` remain
local baseline/tuning output and are not cited as final Task 63 acceptance
evidence.

## Files

- `benchmarks/task63-hnsw-rabitq-format/manifest.md`
- `plan/tasks/63-hnsw-rabitq-storage-format.md`

## Validation

Not run. This is a documentation/metadata-only wording update and does not
change implementation code, benchmark configuration, or benchmark artifacts.
