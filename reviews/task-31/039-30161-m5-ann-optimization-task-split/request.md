# M5 ANN Optimization Task Split

## Scope

This is a docs-only planning checkpoint. It creates three separate follow-up
tasks for the new M5 local optimization lane, in the requested priority order:

1. IVF
2. DiskANN
3. HNSW

No runtime code changes and no measurement claims are introduced here.

## Files

- `plan/tasks/31-ivf-m5-optimization.md`
- `plan/tasks/32-diskann-m5-optimization.md`
- `plan/tasks/33-hnsw-m5-optimization.md`
- `plan/tasks/README.md`

## Planning Summary

- Task 31 makes IVF the first M5 target and emphasizes baseline refresh,
  bottleneck classification, posting-list scan planning, score-as-you-read,
  rerank policy, and churn measurement.
- Task 32 makes DiskANN the second target and starts from the landed Task 29d
  profile, with low-L scan latency and recall-preserving constant-factor work
  as the main focus.
- Task 33 makes HNSW the third target and preserves the Task 26 conclusion:
  refresh worker curves first, then choose direct DSM ingestion,
  offline/staged bulk build, or narrow scan/build hot-path work.

## Validation

- `git diff --check`

Tests were not run because this checkpoint only adds planning documentation.

## Review Focus

- Does the priority order match the requested IVF, DiskANN, HNSW sequence?
- Are the three tasks independently actionable enough for separate follow-up
  packets?
- Are the M5 local measurements clearly separated from future product-class
  benchmark claims?
