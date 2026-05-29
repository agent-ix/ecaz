# Task 63 HNSW RaBitQ Suite Index Cleanup

## Summary

This metadata-only packet updates the authoritative Task 63 handoff references
after adding the checked-in m5 laptop suite config in packet 020.

Changes:

- `plan/tasks/63-hnsw-rabitq-storage-format.md` now points the remaining
  faster-host gate at both checked-in configs:
  - `benchmarks/task63-hnsw-rabitq-format/suite.json` for newer Intel/Linux
  - `benchmarks/task63-hnsw-rabitq-format/suite-m5.json` for m5 laptop
- `benchmarks/task63-hnsw-rabitq-format/manifest.md` now states that Linux
  stages under `/var/lib/pgsql/18/datasets/staged-task63-hnsw-rabitq/`, while
  m5 reuses `data/task31_m5_dbpedia_staged/`.
- The task/manifest follow-up packet index now includes packets through
  `reviews/task-63/020-hnsw-rabitq-m5-suite-config/`.

No benchmarks were run and no benchmark artifacts were changed.

## Validation

Static metadata review only. The earlier packet 020 validation covers
`suite-m5.json` JSON parsing and HNSW-only 50k/100k scope.
