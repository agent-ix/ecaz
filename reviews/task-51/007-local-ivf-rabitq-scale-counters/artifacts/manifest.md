# Artifact Manifest: Local IVF/RaBitQ Scale And Counters

- task bucket: `reviews/task-51/007-local-ivf-rabitq-scale-counters`
- timestamp: `2026-05-23T06:05:36Z`
- benchmark packet: `benchmarks/task51-local-ivf-rabitq-scale`
- benchmark packet commit: `00cfaf0e9` (`Add Task 51 local IVF RaBitQ scale suite`)
- benchmark head SHA at packet write: `807d2389a5f1bd5d128fea2fe67ba27e15b4b891`
- lane: local IVF/RaBitQ geometry scale-up with counters
- fixtures: DBpedia real 50k and 100k
- storage format: `rabitq`
- rerank mode: `heap_f32`
- rerank width: 50
- isolated one-index-per-table surfaces: yes
- AWS: not used
- vchord: not run
- pgvectorscale: not run

## Artifacts

- `benchmarks/task51-local-ivf-rabitq-scale/suite.json`
  - checked-in `SuiteConfig`
- `benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-manifest.json`
  - structured suite manifest
- `benchmarks/task51-local-ivf-rabitq-scale/artifacts/results.jsonl`
  - structured results, 100 rows
- `benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-run.log`
  - final successful suite run
- `benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-status.log`
  - result: `completed=20 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-report.log`
  - parsed load, recall, latency, storage, and planner-cost report
- `benchmarks/task51-local-ivf-rabitq-scale/artifacts/explain-*-rabitq1-n*-w50.log`
  - EXPLAIN counter logs
- `benchmarks/task51-local-ivf-rabitq-scale/manifest.md`
  - benchmark packet provenance and interpretation
- `diff-check.log`
  - command: `git diff --check -- benchmarks/task51-local-ivf-rabitq-scale reviews/task-51/007-local-ivf-rabitq-scale-counters`
  - result: passed

## Notes

This packet addresses methodology feedback from packet 006, but it does not
close Task 51. It is local-only evidence. AWS remains the final gate after
local Experiment 7 sidecar work and any reviewer feedback are processed.
