# Review Request: HNSW RaBitQ Docs Caveat

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- packet: `reviews/task-63/012-hnsw-rabitq-docs-caveat/`

## Summary

This packet updates user-facing docs so they match the current Task 63 state:
HNSW RaBitQ is implemented, but the final operating-point decision is still
benchmark-gated.

## Touched Files

- `docs/usage.md`
  - lists HNSW `storage_format = 'rabitq'` as available but
    benchmark-gated;
  - points production-style HNSW comparisons at TurboQuant/PqFastScan until
    Task 63 records the publishable 50k/100k recommend/shelve decision.
- `README.md`
  - updates the quantization and access-method summary tables with the same
    caveat.

## Validation

No tests or benchmarks were run. This is documentation-only and is intended to
prevent readers from mistaking the implemented HNSW RaBitQ surface for a final
recommended operating point.
