# Task 167 packet 017 manifest

- head SHA: `3b0c74358`
- task bucket: `reviews/task-167/`
- packet: `reviews/task-167/017-insert-work-counters/`
- lane: PG18 physical-generation incremental insert instrumentation
- fixture: physical-vs-local single-row insert throughput A/B
- scales: suite config remains 10k, 50k, 100k in packet 016
- storage format: Task 179 physical generation format
- rerank mode: co-located row-tier exact rerank
- command: see `artifacts/validation.log`; the suite config was syntax-validated
- timestamp: `2026-08-11` America/Los_Angeles
- shared-table/isolated-table surface: each suite step is an isolated cluster;
  the insert A/B uses physical and local control tables within that step

The checkpoint adds six benchmark-only insert-work metrics and emits their
measured values and mean per insert beside throughput. Runtime execution and
the required 10k/50k/100k results remain pending on a benchmark host.
