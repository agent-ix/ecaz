# Review Request: HNSW RaBitQ Benchmark Handoff

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- commits:
  - `7aa55ed5f838acab1b119434e232af6243b7cd6a` - Clarify Task 63 benchmark handoff
  - `d0490c94b2527e037a20342050694d21eabb210e` - Clarify Task 63 benchmark source head
  - `8751a0fa250a3a2286831c5895832cf92b930eb1` - Update HNSW RaBitQ task status
- branch: `task/60-diskann-rabitq`
- packet: `reviews/task-63/011-hnsw-rabitq-benchmark-handoff/`

## Summary

This packet makes the current Task 63 state explicit after the implementation
and local tuning packets:

- HNSW `storage_format = 'rabitq'` implementation work has landed.
- The checked-in benchmark suite remains the canonical publishable Task 63
  measurement surface.
- Older AMD-local benchmark artifacts are baseline/tuning evidence only and
  must not be used as final acceptance evidence.
- Final Task 63 completion still requires newer-host post-scorer 50k/100k
  HNSW-only evidence and a durable recommend/experimental/shelve decision.

## Touched Files

- `benchmarks/task63-hnsw-rabitq-format/manifest.md`
  - records the minimum code source head for publishable runs:
    `36807d607606808717e0b645cde9b251d3fa2e23`;
  - adds a decision-row template;
  - states the criterion for recommending or shelving RaBitQ HNSW;
  - excludes older AMD-local output from final acceptance.
- `plan/tasks/63-hnsw-rabitq-storage-format.md`
  - updates task status to implementation landed with publishable benchmark
    decision pending;
  - lists the current implementation evidence and the remaining completion gate.

## Validation

No benchmark or test command was run for this packet. This is a review-visible
handoff/status packet only, created after the tracked manifest and task-status
changes were committed and pushed.

The local untracked files under
`benchmarks/task63-hnsw-rabitq-format/artifacts/` were intentionally left
untouched.
